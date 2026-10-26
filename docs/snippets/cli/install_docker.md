```bash title="Bash"
docker pull ghcr.io/kreuzberg-dev/kreuzberg-cli:latest
docker run -v $(pwd):/data ghcr.io/kreuzberg-dev/kreuzberg-cli:latest extract /data/document.pdf
```
