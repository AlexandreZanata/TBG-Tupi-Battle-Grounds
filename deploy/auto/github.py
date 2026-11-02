"""Read public GitHub releases tokenlessly with bounded, cached and quota-logged requests."""
import hashlib
import json
import re
import sys
from urllib.error import HTTPError
from urllib.parse import urlparse
from urllib.request import HTTPRedirectHandler, Request, build_opener

from activate import STATE, write_json
from policy import REPOSITORY, require

API = 'https://api.github.com/repos/'+REPOSITORY
REPOSITORY_ID = 1285090346
PAGE = 100
CACHE = STATE/'api-cache'


class HttpsRedirects(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        parsed = urlparse(newurl)
        require(parsed.scheme == 'https' and parsed.hostname in {
            'api.github.com', 'github.com', 'release-assets.githubusercontent.com', 'objects.githubusercontent.com',
        }, 'unexpected download redirect')
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def request(url, maximum, headers=None):
    """(status, headers, body); a 304 is returned, not raised."""
    require(url.startswith('https://'), 'HTTPS required')
    wanted = {'User-Agent': 'TBG-main-deploy', 'Accept': 'application/vnd.github+json', **(headers or {})}
    try:
        with build_opener(HttpsRedirects()).open(Request(url, headers=wanted), timeout=30) as response:
            data = response.read(maximum+1)
            status, meta = response.status, response.headers
    except HTTPError as error:
        if error.code != 304:
            raise
        status, meta, data = 304, error.headers, b''
    require(len(data) <= maximum, 'response exceeds size limit')
    return status, meta, data


def fetch(url, maximum):
    return request(url, maximum)[2]


def api(path):
    """Conditional GET: an unchanged answer is a 304, which does not spend the anonymous quota."""
    key = CACHE/(hashlib.sha256(path.encode()).hexdigest()[:32]+'.json')
    cached = json.loads(key.read_text()) if key.exists() else None
    headers = {'If-None-Match': cached['etag']} if cached and cached.get('etag') else {}
    status, meta, data = request(API+path, 4 * 1024 * 1024, headers)
    # Quota is measured, not assumed: every controller run logs what it spent.
    print(f'github-api {path or "/"} {status} remaining={meta.get("X-RateLimit-Remaining", "?")}', file=sys.stderr)
    if status == 304:
        require(cached is not None, 'unexpected 304 without a cached answer')
        return json.loads(cached['body'])
    if meta.get('ETag'):
        CACHE.mkdir(mode=0o700, exist_ok=True)
        write_json(key, {'etag': meta['ETag'], 'body': data.decode()})
    return json.loads(data)


def releases():
    require(api('')['id'] == REPOSITORY_ID, 'repository identity changed')
    # The newest page holds every current candidate; the active native tag is
    # revalidated by name, so older releases never silently drop out of view.
    return api(f'/releases?per_page={PAGE}')


def candidate(prefix='server', listing=None):
    require(prefix in {'server', 'web', 'static'}, 'unknown release family')
    listing = releases() if listing is None else listing
    options = [r for r in listing if not r.get('draft') and re.fullmatch(prefix + r'-[0-9a-f]{40}', r.get('tag_name', ''))]
    return max(options, key=lambda r: r['published_at'], default=None)
