#!/bin/bash

# build
trunk build --release

# build if the site will be reachable under "/subdir/", not "/"
#trunk build --release --public-url="/subdir/"

# deploy via rsync
#rsync -rv dist/* user@host:/somewhere/www/

# deploy via Netlify CLI
trunk build --release
netlify deploy --prod
