#!/usr/bin/env python3
"""Compare corresponding PNG pages by exact decoded RGBA8 equality."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import warnings

import PIL
from PIL import Image

HASH = re.compile(r'[0-9a-f]{64}\Z')
MAX_PIXELS = 25_000_000


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load_image(path):
    """Decode once from hashed bytes; reject profiles requiring interpretation."""
    import io
    data = Path(path).read_bytes()
    with warnings.catch_warnings():
        warnings.simplefilter('error', Image.DecompressionBombWarning)
        with Image.open(io.BytesIO(data)) as source:
            if source.format != 'PNG' or source.mode not in ('RGB', 'RGBA'):
                raise ValueError('Only RGB8/RGBA8 PNG inputs are supported; no palette or color conversion.')
            if source.width * source.height > MAX_PIXELS:
                raise ValueError('Image exceeds the 25 million pixel bound.')
            if getattr(source, 'n_frames', 1) != 1:
                raise ValueError('Animated/multiframe PNG inputs are not supported.')
            # PNG IHDR bit depth: Pillow can expose 16-bit RGB as mode RGB while
            # reducing precision, so require the encoded depth before decoding.
            if data[24] != 8:
                raise ValueError('PNG must contain 8-bit samples; precision reduction is forbidden.')
            if 'transparency' in source.info:
                raise ValueError('Color-key transparency is unsupported; supply an explicit RGBA PNG.')
            if source.getexif().get(274, 1) != 1:
                raise ValueError('Nonidentity EXIF orientation is unsupported; no automatic rotation.')
            profile = {key: source.info.get(key) for key in ('gamma', 'srgb', 'chromaticity')}
            profile['icc_profile_sha256'] = digest(source.info['icc_profile']) if 'icc_profile' in source.info else None
            source.load()
            rgba = source.convert('RGBA').tobytes()  # RGB adds opaque alpha only.
            return {'width': source.width, 'height': source.height, 'rgba': rgba,
                    'image_sha256': digest(data), 'pixel_sha256': digest(rgba),
                    'encoded_mode': source.mode, 'color_metadata': profile}


def verify_provenance(provenance, reference, candidate):
    if not isinstance(provenance, dict) or type(provenance.get('schema_version')) is not int or provenance['schema_version'] != 1:
        raise ValueError('Provenance schema_version must be 1.')
    for field in ('case_id', 'profile'):
        if not isinstance(provenance.get(field), str) or not provenance[field].strip():
            raise ValueError('Provenance needs a nonempty ' + field + '.')
    if not isinstance(provenance.get('source_sha256'), str) or not HASH.fullmatch(provenance['source_sha256']):
        raise ValueError('Provenance needs a SHA256 source digest.')
    if type(provenance.get('page_index')) is not int or provenance['page_index'] < 0:
        raise ValueError('Provenance needs a zero-based integer page_index.')
    for role, actual in (('reference', reference), ('candidate', candidate)):
        record = provenance.get(role)
        if not isinstance(record, dict):
            raise ValueError('Provenance needs a ' + role + ' object.')
        if record.get('image_sha256') != actual['image_sha256']:
            raise ValueError(role + ' encoded image SHA256 does not match provenance.')
        if 'pixel_sha256' in record and record['pixel_sha256'] != actual['pixel_sha256']:
            raise ValueError(role + ' decoded RGBA pixel SHA256 does not match provenance.')
        if record.get('source_sha256') != provenance['source_sha256']:
            raise ValueError(role + ' source digest differs from the compared case.')
        if not isinstance(record.get('rasterizer_binary_sha256'), str) or not HASH.fullmatch(record['rasterizer_binary_sha256']):
            raise ValueError(role + ' needs rasterizer_binary_sha256.')
        if not isinstance(record.get('rasterizer_version_and_build'), str) or not record['rasterizer_version_and_build'].strip():
            raise ValueError(role + ' needs rasterizer_version_and_build.')
        argv = record.get('raster_argv')
        if not isinstance(argv, list) or not argv or any(not isinstance(s, str) or not s for s in argv):
            raise ValueError(role + ' needs nonempty raster_argv strings.')
    for field in ('rasterizer_version_and_build', 'rasterizer_binary_sha256', 'raster_argv'):
        if provenance['reference'][field] != provenance['candidate'][field]:
            raise ValueError('Rasterizer settings differ: ' + field + '. Use identical version/build and argument template.')
    if reference['color_metadata'] != candidate['color_metadata']:
        raise ValueError('PNG color metadata differs; incompatible color/profile interpretation is rejected.')


def compare(reference, candidate, provenance, *, region_size=64, max_regions=100):
    """Exact acceptance is fixed; region options change diagnostic grouping only."""
    if type(region_size) is not int or region_size < 1 or type(max_regions) is not int or max_regions < 1:
        raise ValueError('Region size and maximum region count must be positive integers.')
    verify_provenance(provenance, reference, candidate)
    result = {
        'schema_version': 1, 'case_id': provenance['case_id'], 'profile': provenance['profile'],
        'page_index': provenance['page_index'], 'source_sha256': provenance['source_sha256'],
        'acceptance': 'exact decoded RGBA8 equality of the entire corresponding page',
        'decoder': {'name': 'Pillow', 'version': PIL.__version__},
        'pixel_hash_format': 'SHA256 of row-major RGBA8 bytes; width and height are checked separately',
        'provenance': provenance,
        'reference': {k: v for k, v in reference.items() if k != 'rgba'},
        'candidate': {k: v for k, v in candidate.items() if k != 'rgba'},
        'bounds_convention': '[left, top, right, bottom], zero-based pixels, right/bottom exclusive',
        'region_method': 'fixed square tiles in original page coordinates; diagnostics only',
        'region_size': region_size, 'max_regions': max_regions,
        'adjustments_applied': [], 'required_max_channel_delta': 0,
        'required_differing_pixel_count': 0}
    width, height = reference['width'], reference['height']
    if ((width + region_size - 1) // region_size) * ((height + region_size - 1) // region_size) > 100_000:
        raise ValueError('Diagnostic grid exceeds 100000 tiles; increase region_size without changing exact acceptance.')
    dimensions_equal = (width, height) == (candidate['width'], candidate['height'])
    if not dimensions_equal:
        overlap = min(width, candidate['width']) * min(height, candidate['height'])
        result.update(status='fail', exact_equal=False, dimensions_equal=False,
                      differing_pixel_count=None, max_channel_delta=None, mismatch_bbox=None,
                      missing_pixel_count=width * height + candidate['width'] * candidate['height'] - 2 * overlap,
                      overlap_pixels_compared=False, regions=[], region_count=0,
                      dimensions_diagnostic={'reference_bounds': [0, 0, width, height],
                                             'candidate_bounds': [0, 0, candidate['width'], candidate['height']]})
        return result
    left, top, right, bottom = width, height, 0, 0
    differing = 0
    max_delta = 0
    channels = [0, 0, 0, 0]
    regions = {}
    a, b = reference['rgba'], candidate['rgba']
    if a != b:
        for pixel in range(width * height):
            offset = pixel * 4
            first, second = a[offset:offset + 4], b[offset:offset + 4]
            if first == second:
                continue
            differing += 1
            x, y = pixel % width, pixel // width
            left, top, right, bottom = min(left, x), min(top, y), max(right, x + 1), max(bottom, y + 1)
            deltas = [abs(first[c] - second[c]) for c in range(4)]
            max_delta = max(max_delta, *deltas)
            for channel in range(4):
                channels[channel] += deltas[channel] != 0
            key = (y // region_size, x // region_size)
            region = regions.setdefault(key, {'tile': [key[1], key[0]], 'differing_pixel_count': 0,
                                               'bbox': [x, y, x + 1, y + 1], 'max_channel_delta': 0})
            region['differing_pixel_count'] += 1
            box = region['bbox']
            region['bbox'] = [min(box[0], x), min(box[1], y), max(box[2], x + 1), max(box[3], y + 1)]
            region['max_channel_delta'] = max(region['max_channel_delta'], *deltas)
    ordered = [regions[key] for key in sorted(regions)]
    result.update(status='pass' if differing == 0 else 'fail', exact_equal=differing == 0,
                  dimensions_equal=True, differing_pixel_count=differing, max_channel_delta=max_delta,
                  differing_channel_counts=dict(zip(('red', 'green', 'blue', 'alpha'), channels)),
                  mismatch_bbox=[left, top, right, bottom] if differing else None,
                  missing_pixel_count=0, overlap_pixels_compared=True,
                  region_count=len(ordered), regions=ordered[:max_regions],
                  regions_truncated=len(ordered) > max_regions)
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--provenance', type=Path, required=True)
    parser.add_argument('--region-size', type=int, default=64)
    parser.add_argument('--max-regions', type=int, default=100)
    args = parser.parse_args(argv)
    try:
        provenance_bytes = args.provenance.read_bytes()
        provenance = json.loads(provenance_bytes)
        result = compare(load_image(args.reference), load_image(args.candidate), provenance,
                         region_size=args.region_size, max_regions=args.max_regions)
        result['provenance_file_sha256'] = digest(provenance_bytes)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0 if result['exact_equal'] else 1
    except (OSError, ValueError, Image.DecompressionBombError, Image.DecompressionBombWarning) as exc:
        print(json.dumps({'schema_version': 1, 'status': 'error', 'exact_equal': False,
                          'error': str(exc)}, sort_keys=True))
        return 2


if __name__ == '__main__':
    sys.exit(main())
