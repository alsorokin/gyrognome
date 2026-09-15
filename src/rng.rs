#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AleaState(pub [f64; 4]);

#[derive(Debug, Clone)]
pub struct Alea {
    s0: f64,
    s1: f64,
    s2: f64,
    c: f64,
}

impl Alea {
    pub fn from_state(state: AleaState) -> Self {
        Self {
            s0: state.0[0],
            s1: state.0[1],
            s2: state.0[2],
            c: state.0[3],
        }
    }

    pub fn state(&self) -> AleaState {
        AleaState([self.s0, self.s1, self.s2, self.c])
    }

    pub fn next_fraction(&mut self) -> f64 {
        let t = 2_091_639.0 * self.s0 + self.c * 2.328_306_436_538_696_3e-10;
        self.s0 = self.s1;
        self.s1 = self.s2;
        self.c = t.floor();
        self.s2 = t - self.c;
        self.s2
    }

    pub fn uint32(&mut self) -> f64 {
        self.next_fraction() * 4_294_967_296.0
    }

    pub fn bounded(&mut self, upper_bound: u32) -> f64 {
        assert!(
            upper_bound > 0,
            "bounded random upper bound must be positive"
        );
        self.uint32() % f64::from(upper_bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_exact_continuation() {
        let mut uninterrupted = Alea::from_state(AleaState([0.1, 0.2, 0.3, 1.0]));
        for _ in 0..5 {
            uninterrupted.next_fraction();
        }
        let state = uninterrupted.state();
        let expected: Vec<_> = (0..5).map(|_| uninterrupted.uint32()).collect();
        let mut restored = Alea::from_state(state);
        assert_eq!(
            expected,
            (0..5).map(|_| restored.uint32()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn matches_browser_reference_values() {
        let mut rng = Alea::from_state(AleaState([0.1, 0.2, 0.3, 1.0]));
        assert_eq!(
            (0..5).map(|_| rng.uint32()).collect::<Vec<_>>(),
            vec![
                3_865_470_567.5,
                3_436_183_000.0,
                3_006_895_434.0,
                432_425_023.0,
                268_118_227.0
            ]
        );
    }
}
