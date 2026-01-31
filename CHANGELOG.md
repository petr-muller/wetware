# Changelog

## [0.4.0](https://github.com/petr-muller/wetware/compare/v0.3.0...v0.4.0) (2024-11-10)


### Features

* **edit:** allow editing thought dates ([#102](https://github.com/petr-muller/wetware/issues/102)) ([aa2b652](https://github.com/petr-muller/wetware/commit/aa2b65290a58162367ee4bb942840dc683b2607f))
* **edit:** allow editing thoughts ([#105](https://github.com/petr-muller/wetware/issues/105)) ([e2f8e94](https://github.com/petr-muller/wetware/commit/e2f8e94591a081b90212b430407328ca2c948f1e))
* **thought:** do not emit hours ([#97](https://github.com/petr-muller/wetware/issues/97)) ([8daaec1](https://github.com/petr-muller/wetware/commit/8daaec195d146d10e5f1a3ba73578281af81aba6))
* **thought:** expose thought IDs ([#100](https://github.com/petr-muller/wetware/issues/100)) ([e52dcb5](https://github.com/petr-muller/wetware/commit/e52dcb5ee1ac02dacaf9e73438abfa63527154c8))
* **thoughts:** allow aliased entity references ([#94](https://github.com/petr-muller/wetware/issues/94)) ([10fdff3](https://github.com/petr-muller/wetware/commit/10fdff3ce6edaa02b142da229f67fd4b9f954548))
* **thoughts:** use dates without times everywhere ([#98](https://github.com/petr-muller/wetware/issues/98)) ([1b62bf6](https://github.com/petr-muller/wetware/commit/1b62bf632d917ab7016608ac0fe49c043a808a1e))
* thoughts and tui share backend ([#93](https://github.com/petr-muller/wetware/issues/93)) ([8d0cb95](https://github.com/petr-muller/wetware/commit/8d0cb958a3faa59f833611437099a1e8a1f0b4f9))
* **tui:** extract tui to module, pass tests ([#89](https://github.com/petr-muller/wetware/issues/89)) ([3f91205](https://github.com/petr-muller/wetware/commit/3f9120548e66cb1ceee9e5a4ba500be1758ee695))
* **tui:** simple persistent entity->color mapper ([#85](https://github.com/petr-muller/wetware/issues/85)) ([cf032f0](https://github.com/petr-muller/wetware/commit/cf032f01f068e492f0cf405bb22336004184b8ec))


### Bug Fixes

* **add:** multiple refs from one thought to one entity ([#104](https://github.com/petr-muller/wetware/issues/104)) ([4353188](https://github.com/petr-muller/wetware/commit/4353188e6cf391ed9d154e3b53f05befb15d8e5b))

## [0.3.0](https://github.com/petr-muller/wetware/compare/v0.2.0...v0.3.0) (2024-03-29)


### Features

* improve thought processing and detect errors ([#30](https://github.com/petr-muller/wetware/issues/30)) ([11e58d1](https://github.com/petr-muller/wetware/commit/11e58d1fbb71999c929395820c58488d65e01729))
* **wet entities:** new `wet entities` command to list entities ([#53](https://github.com/petr-muller/wetware/issues/53)) ([fd00577](https://github.com/petr-muller/wetware/commit/fd0057708fa0b470bab110095be52a81b1bd9726))
* **wet thoughts:** --on=entity ([#23](https://github.com/petr-muller/wetware/issues/23)) ([dd92c5d](https://github.com/petr-muller/wetware/commit/dd92c5d5b2c0cf016a4cfb666b4665e093241d8c))
* **wet thoughts:** simple command ([#21](https://github.com/petr-muller/wetware/issues/21)) ([05498c3](https://github.com/petr-muller/wetware/commit/05498c39293f12894bb6909b1a98483fee8cba6f))


### Bug Fixes

* improve command help messages ([d7d96c9](https://github.com/petr-muller/wetware/commit/d7d96c944b0afac527fe7edbad137073c9cd6bb2))
* **wet add:** improve message on bad thought ([#31](https://github.com/petr-muller/wetware/issues/31)) ([c9c1e4e](https://github.com/petr-muller/wetware/commit/c9c1e4eae45fc18c8812e1c99a4be13bfc6aef3e))

## [0.2.0](https://github.com/petr-muller/wetware/compare/v0.1.2...v0.2.0) (2023-11-11)


### Features

* **wet add:** support basic entity links ([#15](https://github.com/petr-muller/wetware/issues/15)) ([0430b43](https://github.com/petr-muller/wetware/commit/0430b43c48750aa27b7aeab53691470ca2998a3b))
* **wet add:** support linking thoughts with dates ([72ae57b](https://github.com/petr-muller/wetware/commit/72ae57bde4e63fb80b1ae90523588b843cd659b5))


### Bug Fixes

* **deps:** bump all dependencies ([#14](https://github.com/petr-muller/wetware/issues/14)) ([72d6b1e](https://github.com/petr-muller/wetware/commit/72d6b1ed945595a6284da661167aec8b8c50b6cb))

## 0.1.2 (2023-08-23)


### Features

* **wet add:** implement trivial sqlite-backed storage ([97b91fa](https://github.com/petr-muller/wetware/commit/97b91fa4efd9f52a0236c706d42a686a62607f82))
* **wet:** scaffold `add` and integration tests ([9b9353b](https://github.com/petr-muller/wetware/commit/9b9353bfc45ead7c66ae3a300f924da513d9315b))
