This is used for cross-compile for ci builds.
See [hub.docker.com/r/rsadsb/ci](https://hub.docker.com/r/rsadsb/ci).

```
docker build -f ./armv7-unknown-linux-gnueabihf.Dockerfile -t rsadsb/ci:0.2.0-armv7-unknown-linux-gnueabihf .
docker build -f ./x86_64-unknown-linux-gnu.Dockerfile -t rsadsb/ci:0.2.0-x86_64-unknown-linux-gnu .
```
