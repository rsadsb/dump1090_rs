# dump1090_rs

Fork of https://github.com/johnwstanford/dump1090_rs, without parsing messages.
This project is meant to just forward bytes from the the demodulated iq to our own [adsb_deku](https://github.com/wcampbell0x2a/adsb_deku)

# Usage

```
cargo r
```

# Changes
- Removed packet parsing functions
- Removed read in from file
- Added TCP forwarding to client

