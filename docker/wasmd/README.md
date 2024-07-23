# Quartz app wasmd image

This folder contains a `Dockerfile` that helps build a single-node [wasmd]
validator for use in testing your Quartz application.

It facilitates the creation of a Docker image with 4 accounts pre-loaded, each
having a small amount of `ucosm` preloaded from genesis for experimentation.

- `admin`
- `alice`
- `bob`
- `charlie`

**Note: this image is _not_ intended to be used in production.**

## Building the image

Simply build the image using the supplied `Makefile`:

```bash
make
```

By default, this will build an image tagged `informaldev/wasmd:v0.44.0`.

[wasmd]: https://github.com/CosmWasm/wasmd
