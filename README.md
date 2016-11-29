# buoyancy

This is a small crate that implements a fast algorithm for float placement according to the rules
in CSS 2.1 § 9.5.1.

The cornerstone of the technique is a splay tree, which effectively accelerates the common case of
placing an object right next to the most recent floats that were placed. Randomized testing
suggests that, in practice, the algorithm places n floats in O(n) time. Even when an object ends up
placed in more distant positions, the splay tree achieves O(log n) randomized performance. In the
worst case, this algorithm is O(n²); however, this seems to be very rare in practice.

On my MacBook Pro with a 2.8 GHz Intel Core i7, this implementation places each float in 6 µs,
approximately 160,000 floats per second.

The splay tree implementation is a modified version of
[splay-rs](https://github.com/alexcrichton/splay-rs).

## License

Licensed under the same terms as Rust itself.

