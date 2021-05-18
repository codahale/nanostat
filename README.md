# nanostat

Like ministat, but smaller?

A Rust library and CLI tool for evaluating whether two or more sets of measurements are statistically different. It does
this by performing a *Welch's t-test* at a particular confidence level, making it suitable for small sets of
measurements (e.g., multiple runs of a benchmark). It's inspired largely by FreeBSD's `ministat` (written by
Poul-Henning Kamp).

```
$ nanostat examples/iguana examples/leopard examples/chameleon 
examples/leopard:
	Difference at 95% confidence!
		643.50 > 300.00 ± 293.97, p = .026
examples/chameleon:
	No difference at 95% confidence.
```

## Install

```
cargo install nanostat --features cli
```

## License

Copyright © 2021 Coda Hale

Distributed under the Apache License 2.0 or the MIT license.
