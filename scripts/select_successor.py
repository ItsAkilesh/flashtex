#!/usr/bin/env python3
"""Pure, bounded successor recommendation; never grants authority or spends usage."""
import argparse
from datetime import datetime, timezone
import json
import math
from pathlib import Path


def select(records, now, preferred='linux-primary', max_age=300):
    if not isinstance(records, list) or len(records) > 128:
        raise ValueError('bounded candidate list required')
    eligible, excluded = [], []
    seen = set()
    for r in records:
        machine = r.get('machine')
        if not isinstance(machine, str) or not machine or machine in seen:
            raise ValueError('unique machine identities required')
        seen.add(machine)
        reason = None
        try:
            stamp = datetime.fromisoformat(r['observed_utc'].replace('Z', '+00:00'))
            age = (now - stamp).total_seconds()
            if not 0 <= age <= max_age:
                reason = 'stale_or_future_evidence'
        except (KeyError, ValueError, TypeError):
            reason = 'invalid_timestamp'
        if not all(r.get(k) is True for k in ('registered', 'working_verified', 'billing_authorized', 'orchestration_capable', 'usable_capacity_verified')):
            reason = 'missing_verified_eligibility'
        if not isinstance(r.get('session_evidence'), str) or not r['session_evidence']:
            reason = 'missing_session_evidence'
        if reason:
            excluded.append({'machine': machine, 'reason': reason})
        else:
            eligible.append(r)
    result = {'selected': None, 'claim_authorized': False, 'excluded': excluded}
    if any(r['machine'] == preferred for r in eligible):
        result.update(selected=preferred, reason='preferred_host_still_usable')
        return result
    ranked = []
    for r in eligible:
        n, unit = r.get('remaining'), r.get('comparison_profile')
        if type(n) not in (int, float) or not math.isfinite(n) or n <= 0 or not isinstance(unit, str) or not unit:
            result['excluded'].append({'machine': r['machine'], 'reason': 'unknown_remaining_capacity'})
        else:
            ranked.append(r)
    if not ranked:
        result['reason'] = 'no_verified_remaining_capacity'
    elif len({r['comparison_profile'] for r in ranked}) != 1:
        result['reason'] = 'incomparable_capacity_units_require_review'
    else:
        winner = sorted(ranked, key=lambda r: (-r['remaining'], r['machine']))[0]
        result.update(selected=winner['machine'], reason='greatest_verified_comparable_remaining_capacity', evidence=winner)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('evidence')
    args = parser.parse_args()
    path = Path(args.evidence)
    if path.stat().st_size > 1048576:
        raise SystemExit('evidence exceeds 1MiB')
    print(json.dumps(select(json.loads(path.read_text()), datetime.now(timezone.utc)), indent=2))
