# Special bindgen generation

## Quick start

From the top level directory, run:

```
env ICU_SOURCE_DIR=$HOME/code/icu make static-bindgen-special
```

This assumes that `$HOME/code/icu` contains an ICU library checkout.

## What does this do?

This runs an extended version of ../run_bindgen.sh, which generates the API
not only for the public ICU files, but also for the internal tools.
