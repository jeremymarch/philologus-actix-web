## philolog-us-actix-web

A new version of the philolog.us server rewritten in Rust.  

Set the path to the sqlite database as an environment variable:

export PHILOLOGUS_DB_PATH=sqlite://philolog_us_local.sqlite?mode=ro

A truncated sample database is provided for testing.


To run eslint on Javascript code: ./node_modules/.bin/eslint --ext .html .

Load lexica into the sqlite db and tantivy full-text index with [philologus-lex-loader](https://github.com/jeremymarch/philologus-lex-loader).
