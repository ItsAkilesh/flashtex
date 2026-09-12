#!/usr/bin/env python3
"""Validate visual sources and declared reference artifacts, never visual fidelity.

Usage: python3 scripts/check_visual_manifest.py MANIFEST [--artifact-root DIR]
Exit 0: source/provenance fields valid (references may be pending); 1: invalid.
Generated RGB PNG pixel-hash verification uses Pillow. Missing Pillow/artifacts
remain pending. Font/engine provenance is operator-attested; hashes verify bytes,
not that a reference engine actually executed or that the candidate matches it.
"""
import argparse
from datetime import datetime
import hashlib
import json
import math
from pathlib import Path
import re
import sys

HASH = re.compile(r'^[0-9a-f]{64}$')


class Invalid(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise Invalid(message)


def text(value, field):
    require(isinstance(value, str) and bool(value.strip()), field + ' must be nonempty text')
    return value


def digest(value):
    require(isinstance(value, str) and bool(HASH.fullmatch(value)), 'invalid SHA-256')
    return value


def safe_file(root, name):
    text(name, 'path')
    path = Path(name)
    require(not path.is_absolute() and '\\' not in name and '..' not in path.parts, 'unsafe relative artifact/source path')
    resolved = (root / path).resolve()
    require(resolved.is_relative_to(root.resolve()), 'path escapes root')
    return resolved


def file_check(root, entry, pending):
    path = safe_file(root, entry['path']); expected = digest(entry['sha256'])
    if not path.is_file():
        pending.append('missing artifact: ' + entry['path']); return None
    require(hashlib.sha256(path.read_bytes()).hexdigest() == expected, 'artifact hash mismatch: ' + entry['path'])
    return path


def profile_check(profile):
    text(profile['reference_engine'], 'reference_engine'); text(profile['font_policy'], 'font_policy')
    text(profile['engine_version'], 'engine_version policy'); text(profile['rasterizer_version'], 'rasterizer_version policy')
    require(isinstance(profile['required_packages'], list) and bool(profile['required_packages']), 'required packages missing')
    for package in profile['required_packages']: text(package, 'package')
    dpi = profile['dpi']; box = profile['page_media_box_bp']; pixels = profile['expected_page_pixels']
    require(type(dpi) in (int, float) and math.isfinite(dpi) and dpi > 0, 'invalid DPI')
    require(isinstance(box, list) and len(box) == 4 and all(type(n) in (int, float) and math.isfinite(n) for n in box), 'invalid MediaBox')
    require(box[2] > box[0] and box[3] > box[1], 'empty MediaBox')
    require(isinstance(pixels, list) and len(pixels) == 2 and all(type(n) is int and n > 0 for n in pixels), 'invalid page pixel dimensions')
    require(pixels == [math.ceil((box[2]-box[0])*dpi/72), math.ceil((box[3]-box[1])*dpi/72)], 'DPI/MediaBox/pixel dimensions disagree')
    comparison = profile['comparison']
    text(comparison['primary'], 'comparison primary gate')
    require(isinstance(comparison['secondary_diagnostics'], list), 'secondary diagnostics must be a list')
    for key in ('required_max_channel_delta', 'required_differing_pixel_count'):
        require(type(comparison[key]) is int and comparison[key] == 0, 'exact pixel comparison requires zero tolerance')
    require(comparison['secondary_metrics_are_not_a_pass'] is True, 'secondary metrics cannot replace exact equality')
    require(set(('auto-alignment','rescaling','cropping','blur','font substitution','threshold relaxation')) <= set(comparison['forbidden_adjustments']), 'pixel-altering adjustments cannot be enabled')
    text(comparison['color_channels'], 'comparison color channels')
    text(profile['rasterizer'], 'rasterizer')
    require(isinstance(profile['raster_argv'], list) and all(isinstance(a, str) for a in profile['raster_argv']), 'raster argv must be strings')


def reference_check(case, profile, root):
    pending = []
    reference = case.get('reference')
    if case['reference_state'] != 'generated':
        return {'status':'pending', 'reasons':['reference not generated; source validity is not measured visual completion']}
    if reference is None:
        return {'status':'pending', 'reasons':['generated reference record missing']}
    require(reference['source_sha256'] == case['source_sha256'], 'reference is stale for fixture source')
    for field in ('engine_version','format_date','generation_host','generation_time_utc'):
        value = text(reference[field], field)
        require(value.strip().lower() not in ('unknown','pending','unverified','not_generated'), 'unmeasured reference provenance: ' + field)
    require(datetime.fromisoformat(reference['generation_time_utc'].replace('Z','+00:00')).tzinfo is not None, 'generation time needs timezone')
    require(reference['engine'] == profile['reference_engine'], 'reference engine profile mismatch')
    packages = reference['package_versions']
    for name in profile['required_packages']: text(packages[name], 'package version ' + name)
    require(isinstance(reference['fonts'], list) and reference['fonts'], 'actual font inventory required')
    for font in reference['fonts']:
        text(font['name'], 'font name')
        require(isinstance(font['files'], list) and {'font_program','metrics'} <= {f['kind'] for f in font['files']}, 'font program and metric hashes required')
        for entry in font['files']: file_check(root, entry, pending)
    render = reference['render']
    require(render['dpi'] == profile['dpi'] and render['media_box_bp'] == profile['page_media_box_bp'] and render['page_pixels'] == profile['expected_page_pixels'], 'reference render geometry differs from profile')
    require(render['color_space'] == 'RGB' and render['background'] == 'white', 'opaque white RGB reference required')
    color = render['color_profile']
    require(color['mode'] in ('unmanaged','icc'), 'explicit color-profile mode required')
    if color['mode'] == 'icc': file_check(root, color['file'], pending)
    else: text(color['description'], 'unmanaged color profile description')
    require(render['argv_template'] == profile['raster_argv'], 'raster parameters differ from profile')
    raster = reference['rasterizer']
    require(raster['name'] == profile['rasterizer'], 'rasterizer mismatch')
    text(raster['version'], 'rasterizer version'); digest(raster['binary_sha256'])
    pdf = file_check(root, reference['pdf'], pending)
    if pdf: require(pdf.read_bytes().startswith(b'%PDF-'), 'reference artifact lacks PDF header')
    pages = reference['pages']
    require(isinstance(pages, list) and len(pages) == case['expected_page_count'], 'reference page count mismatch')
    for number, page in enumerate(pages, 1):
        require(type(page['number']) is int and page['number'] == number, 'pages must be complete and ordered')
        require([page['width'],page['height']] == profile['expected_page_pixels'], 'reference page geometry mismatch')
        digest(page['pixel_sha256']); path = file_check(root, page, pending)
        if path:
            try:
                from PIL import Image
            except ImportError:
                pending.append('Pillow unavailable: decoded pixel hashes not verified'); continue
            with Image.open(path) as image:
                embedded = image.info.get('icc_profile')
                if color['mode'] == 'unmanaged':
                    require(not embedded, 'PNG embeds an undeclared ICC profile')
                elif embedded:
                    require(hashlib.sha256(embedded).hexdigest() == color['file']['sha256'], 'PNG embedded ICC profile mismatch')
                require(image.format == 'PNG' and image.mode == 'RGB' and list(image.size) == profile['expected_page_pixels'], 'page must be opaque RGB PNG with declared dimensions')
                require(hashlib.sha256(image.convert('RGBA').tobytes()).hexdigest() == page['pixel_sha256'], 'decoded pixel hash mismatch')
    return {'status':'pending' if pending else 'verified', 'reasons':pending,
            'scope':'reference byte hashes and recorded provenance only; engine execution/font correctness not independently attested'}


def check_manifest(path, artifact_root=None):
    path = Path(path); root = path.parent; artifacts = Path(artifact_root) if artifact_root else root
    manifest = json.loads(path.read_text()); results=[]
    require(manifest['schema_version'] == 1, 'unsupported manifest version')
    require(isinstance(manifest['profiles'], dict) and manifest['profiles'], 'profiles missing')
    for profile in manifest['profiles'].values(): profile_check(profile)
    require(isinstance(manifest['cases'], list) and manifest['cases'], 'cases missing')
    ids=set()
    for case in manifest['cases']:
        ident=text(case['id'], 'case id'); require(ident not in ids, 'duplicate case id'); ids.add(ident)
        try:
            require(case['profile'] in manifest['profiles'], 'unknown profile')
            source=safe_file(root,case['entry_path']); require(source.is_file(), 'source file missing')
            require(hashlib.sha256(source.read_bytes()).hexdigest()==digest(case['source_sha256']), 'source hash mismatch')
            source.read_bytes().decode('utf-8')
            require(type(case['expected_page_count']) is int and case['expected_page_count']>0, 'expected page count must be positive')
            require(case['reference_state'] in ('not_generated','pending','generated'), 'invalid reference state')
            text(case['candidate_state'], 'candidate_state')
            for field in ('features','comparison_gates'):
                require(isinstance(case[field], list) and case[field] and all(isinstance(v,str) and v.strip() for v in case[field]), 'invalid ' + field)
            reference=reference_check(case,manifest['profiles'][case['profile']],artifacts)
            results.append({'case_id':ident,'source_status':'valid','reference':reference,'visual_fidelity':'pending'})
        except (ValueError,KeyError,TypeError,OSError) as error:
            results.append({'case_id':ident,'status':'invalid','error':str(error),'visual_fidelity':'pending'})
    return {'status':'invalid' if any(r.get('status')=='invalid' for r in results) else 'valid',
            'reference_status':'pending' if any(r.get('reference',{}).get('status')!='verified' for r in results) else 'verified',
            'visual_fidelity':'pending: exact candidate/reference pixel comparison is a separate gate', 'cases':results}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifest',type=Path); parser.add_argument('--artifact-root',type=Path)
    args=parser.parse_args()
    try:
        result=check_manifest(args.manifest,args.artifact_root)
    except (ValueError,KeyError,TypeError,OSError) as error:
        result={'status':'invalid','error':str(error),'visual_fidelity':'pending'}
    print(json.dumps(result,indent=2)); return int(result['status']=='invalid')


if __name__=='__main__':sys.exit(main())
