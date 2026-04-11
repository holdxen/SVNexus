#!/bin/zsh
#


tmpdir="$(mktemp -d)"

export DATABASE_URL=sqlite://$tmpdir/app.db?mode=rwc
sea-orm-cli migrate up

sea-orm-cli generate entity -v --output-dir ./src/entities --database-schema sqlite --entity-format dense # --expanded-format
