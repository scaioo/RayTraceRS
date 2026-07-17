// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! # PCG — Permuted Congruential Generator
//!
//! A fast, statistically high-quality pseudo-random number generator (PRNG)
//! based on the [PCG family](https://www.pcg-random.org/) by Melissa O'Neill.
//!
//! ## Algorithm
//!
//! The generator advances its internal state using a linear congruential step:
//!
//! ```text
//! state = state × 6364136223846793005 + inc
//! ```
//!
//! Output is produced by applying a *XSH-RR* (xorshift + random rotation)
//! permutation to the old state, yielding a 32-bit value with excellent
//! statistical properties from a 64-bit internal state.
//!
//! ## Example
//!
//! ```rust
//! use rstrace::pcg::PCG;
//! let mut rng = PCG::default();
//!
//! let n: u32  = rng.random();        // uniform integer in [0, 2³²)
//! let f: f32  = rng.random_float();  // uniform float  in [0.0, 1.0)
//! ```
//!
//! ## Determinism
//!
//! Two [`PCG`] instances seeded with the same `(init_state, init_seq)` pair
//! produce **identical** output streams, making this suitable for
//! reproducible simulations and tests.

/// A PCG32 pseudo-random number generator.
///
/// Holds the full mutable state needed to produce the next value.
/// Both fields are kept private to enforce correct initialization through
/// [`PCG::new`] or [`PCG::default`].
///
/// The struct derives [`Copy`] intentionally: cloning or checkpointing the
/// generator state is a valid and inexpensive operation (two `u64` copies).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PCG {
    /// Current LCG state. Advanced on every call to [`PCG::random`].
    state: u64,
    /// Stream selector — always odd, encodes the sequence increment.
    /// Different `inc` values produce different streams.
    inc: u64,
}

impl PCG {
    /// Creates a new [`PCG`] instance from an explicit seed pair.
    ///
    /// The two parameters together select both the *starting position* and
    /// the *stream* within the PCG family:
    ///
    /// - `init_state` — seeds the generator's position in its sequence.
    /// - `init_seq` — selects which of the 2⁶³ independent streams to use.
    ///   Two generators with the same `init_state` but different `init_seq`
    ///   values will produce different output streams.
    ///
    /// The constructor performs two warm-up calls to [`PCG::random`] so that
    /// the initial output is not trivially correlated with the seed.
    ///
    /// # Arguments
    ///
    /// * `init_state` — arbitrary 64-bit seed for the state.
    /// * `init_seq` — stream identifier; only the lower 63 bits are used
    ///   (the increment is forced odd: `inc = (init_seq << 1) | 1`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rstrace::pcg::PCG;
    ///
    /// let mut rng = PCG::new(12345, 67890);
    /// let value = rng.random();
    /// ```
    pub fn new(init_state: u64, init_seq: u64) -> Self {
        let mut rng = PCG {
            state: 0,
            inc: (init_seq << 1) | 1,
        };

        rng.random();
        rng.state += init_state;
        rng.random();

        rng
    }

    /// Advances the generator and returns the next pseudo-random `u32`.
    ///
    /// Applies the PCG *XSH-RR* output transformation to the pre-advance
    /// state:
    ///
    /// 1. **Advance** — updates `state` with the LCG recurrence.
    /// 2. **Xorshift** — folds the high bits of the old state downward.
    /// 3. **Rotate** — randomly rotates the result using bits 59–63 of the
    ///    old state as the rotation amount, breaking the LCG's linearity.
    ///
    /// The output is pseudo-random.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rstrace::pcg::PCG;
    ///
    /// let mut rng = PCG::default();
    /// let a = rng.random();
    /// let b = rng.random();
    /// assert_ne!(a, b); // extremely unlikely to collide
    /// ```
    pub fn random(&mut self) -> u32 {
        let oldstate = self.state;
        self.state = oldstate
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.inc);
        let xorshifted = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
        let rot = (oldstate >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Returns the next pseudo-random `f32` in the half-open interval `[0.0, 1.0)`.
    ///
    /// Converts the raw 32-bit output of [`PCG::random`] to a float by
    /// dividing by `2³²`. The result is never exactly `1.0`.
    ///
    /// Note that only 2³² distinct values are possible; for applications
    /// requiring more floating-point resolution, consider combining two
    /// calls or using a dedicated `f64` generator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rstrace::pcg::PCG;
    ///
    /// let mut rng = PCG::default();
    /// let f = rng.random_float();
    /// assert!((0.0..1.0).contains(&f));
    /// ```
    pub fn random_float(&mut self) -> f32 {
        self.random() as f32 / (u32::MAX as f32 + 1.0)
    }

    /// Returns a vector of `len` random floats in `[0, 1)`.
    pub fn n_random_floats(&mut self, len: usize) -> Vec<f32> {
        let mut v: Vec<f32> = vec![];
        for _ in 0..len {
            v.push(self.random_float());
        }
        v
    }
}
impl Default for PCG {
    /// Returns a [`PCG`] seeded with the library defaults:
    /// - `init_state = 42`.
    /// - `init_seq = 54`.
    fn default() -> Self {
        Self::new(42, 54)
    }
}

// ==============================================
// TESTS
// ==============================================

#[cfg(test)]
mod tests {
    use crate::pcg::PCG;

    #[test]
    fn test_known_sequence() {
        let mut pcg = PCG::default();
        pcg.state = 1753877967969059832;
        pcg.inc = 109;
        for expected in [
            2707161783, 2068313097, 3122475824, 2211639955, 3215226955, 3421331566,
        ] {
            assert_eq!(expected, pcg.random())
        }
    }

    #[test]
    /// checks that pcg is deterministic
    fn test_deterministic() {
        let mut a = PCG::new(42, 54);
        let mut b = PCG::new(42, 54);

        for _ in 0..1000 {
            assert_eq!(a.random(), b.random());
        }
    }

    #[test]
    fn test_n_random_floats() {
        let mut one_step_random = PCG::new(42, 54);
        let mut random_generator = PCG::new(42, 54);

        let n = 100;
        let result = random_generator.n_random_floats(n);

        // Dato che i due PCG sono identici, devono produrre gli stessi valori
        // nello stesso ordine
        for i in 0..n {
            assert_eq!(result[i], one_step_random.random_float());
        }
    }
}
