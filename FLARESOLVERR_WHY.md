# Why FlareSolverr doesn't work on the Pi (simple)

**What FlareSolverr does**  
Some sites (like 1337x) show a “checking your browser” or Cloudflare page before the real content. FlareSolverr gets past that by opening the site in a **real Chrome browser** and waiting for the check to finish, then returning the page.

**Why it needs a “screen”**  
Chrome is built to draw to a display. On a server there’s no monitor, so FlareSolverr uses **Xvfb**: a fake screen in memory. It starts Xvfb, then starts Chrome against that fake screen.

**What goes wrong on the Pi**  
In the **official** image (`ghcr.io/flaresolverr/flaresolverr`), when it runs on the Pi (ARM), **Xvfb fails to start** inside the container. So Chrome never runs, FlareSolverr exits on startup and never listens on port 8191.

**What about the Raspberry Pi guide?**
The [guide](https://flaresolverr.com/using-flaresolverr-on-raspberry-pi/) says to use the **LinuxServer** image: `docker pull linuxserver/flaresolverr`. That image **does not exist**: Docker Hub returns "repository does not exist" and the LinuxServer registry returns "denied". So the only image you can actually pull is the official one, and that one hits the Xvfb failure. The guide is outdated or refers to an image that was never published or was removed.


So it’s not about “Docker” or “proxy” settings; the **image’s way of starting a headless Chrome (via Xvfb) doesn’t work on this ARM setup**. Fixing it would mean a different image or build that either gets Xvfb working on ARM or uses a truly headless Chrome path that doesn’t need Xvfb.

**Try this on the Pi (xvfb-run workaround)**  
The app normally starts Xvfb from Python; that can fail or hit a stuck display. Use `xvfb-run` outside the app so the display exists before the app starts, and use a different display number (e.g. 100) to avoid port/lock conflicts:

1. On the Pi: stop and remove any old container, and kill stray Xvfb (optional):  
   `docker rm -f flaresolverr 2>/dev/null; sudo pkill -9 Xvfb 2>/dev/null; true`
2. Run with overridden command and more shared memory:

```bash
docker run -d \
  --name=flaresolverr \
  -p 8191:8191 \
  -e LOG_LEVEL=info \
  --shm-size=1g \
  --restart unless-stopped \
  ghcr.io/flaresolverr/flaresolverr:latest \
  xvfb-run --server-num=100 --server-args="-screen 0 1024x768x24" /usr/local/bin/python -u /app/flaresolverr.py
```

If it still exits, check `docker logs flaresolverr` for a different error (e.g. Chrome crash, permissions).

**Proper Prowlarr setup**  
FlareSolverr is an **indexer proxy**, not a normal HTTP proxy. Do **not** put it in Settings → Proxy (that’s for things like Tor).

1. **Settings → Proxy (global)**  
   Leave **Use Proxy** off (or use Tor there only if you want all indexer traffic through Tor).

2. **Settings → Indexers → Indexer Proxies**  
   Add FlareSolverr here: URL `http://flaresolverr:8191/`. Give it a tag, e.g. **flaresolverr**.

3. **Per indexer**  
   For any indexer that hits Cloudflare (e.g. 1337x), open that indexer and add the same tag (**flaresolverr**). Only those indexers will use FlareSolverr.

4. **Indexer Test can still fail**  
   Prowlarr’s indexer **Test** does a direct request first; if that hits Cloudflare it reports “blocked by CloudFlare” even when FlareSolverr would work on retry ([Prowlarr #1733](https://github.com/Prowlarr/Prowlarr/issues/1733)). **Workaround:** add the **flaresolverr** tag and **Save** the indexer (ignore or skip Test). Real searches and RSS will use FlareSolverr.

5. **Optional: FlareSolverr via Tor**  
   If you want FlareSolverr to fetch pages through Tor, run the container with `-e PROXY_URL=http://tor-proxy:8118`. Then Prowlarr → FlareSolverr → Tor → site.

**What you can do**  
- Use **Tor** as the proxy for 1337x.  
- Run FlareSolverr with the xvfb-run workaround above and point Prowlarr/Radarr at it (e.g. `http://flaresolverr:8191`).  
- Or run **FlareSolverr on an x86 machine** (PC or VPS) where Xvfb works and point Prowlarr/Radarr at it.
