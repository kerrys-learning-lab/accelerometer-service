use crate::Value;

impl Value {
    pub fn add(&self, rhs: &Value) -> Self {
        let mut other = *self;

        *(other.mut_add(rhs))
    }

    pub fn sub(&self, rhs: &Value) -> Self {
        let mut other = *self;

        *(other.mut_sub(rhs))
    }

    pub fn mul(&self, factor: f64) -> Self {
        let mut other = *self;

        *(other.mut_mul(factor))
    }

    pub fn div(&self, denom: f64) -> Self {
        let mut other = *self;

        *(other.mut_div(denom))
    }

    pub fn mut_add(&mut self, rhs: &Value) -> &Self {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
        self.z = self.z + rhs.z;

        self
    }

    pub fn mut_sub(&mut self, rhs: &Value) -> &Self {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
        self.z = self.z - rhs.z;

        self
    }

    pub fn mut_mul(&mut self, factor: f64) -> &Self {
        self.x = self.x * factor;
        self.y = self.y * factor;
        self.z = self.z * factor;

        self
    }

    pub fn mut_div(&mut self, denom: f64) -> &Self {
        self.x = self.x / denom;
        self.y = self.y / denom;
        self.z = self.z / denom;

        self
    }

    pub fn average<'a, I>(vals: I) -> Value
    where
        I: Iterator<Item = &'a Value>,
    {
        let mut count = 0;
        let mut result: Value = Default::default();

        for v in vals {
            result.mut_add(&v);
            count = count + 1;
        }

        *result.mut_div(count as f64)
    }
}

#[cfg(test)]
mod test {
    use crate::Value;

    const TEST_X: f64 = 1.5;
    const TEST_Y: f64 = 2.4;
    const TEST_Z: f64 = 4.2;
    const FACTOR: f64 = 3.14;

    #[test]
    fn add() {
        let uut: Value = Default::default();

        let new_value = uut.add(&Value {
            x: TEST_X,
            y: TEST_Y,
            z: TEST_Z,
        });

        assert_eq!(TEST_X, new_value.x);
        assert_eq!(TEST_Y, new_value.y);
        assert_eq!(TEST_Z, new_value.z);
    }

    #[test]
    fn sub() {
        let uut: Value = Default::default();

        let new_value = uut.sub(&Value {
            x: TEST_X,
            y: TEST_Y,
            z: TEST_Z,
        });

        assert_eq!(TEST_X * -1.0, new_value.x);
        assert_eq!(TEST_Y * -1.0, new_value.y);
        assert_eq!(TEST_Z * -1.0, new_value.z);
    }

    #[test]
    fn mul() {
        let uut: Value = Value {
            x: TEST_X,
            y: TEST_Y,
            z: TEST_Z,
        };

        let new_value = uut.mul(FACTOR);

        assert_eq!(TEST_X * FACTOR, new_value.x);
        assert_eq!(TEST_Y * FACTOR, new_value.y);
        assert_eq!(TEST_Z * FACTOR, new_value.z);
    }

    #[test]
    fn div() {
        let uut: Value = Value {
            x: TEST_X,
            y: TEST_Y,
            z: TEST_Z,
        };

        let new_value = uut.div(FACTOR);

        assert_eq!(TEST_X / FACTOR, new_value.x);
        assert_eq!(TEST_Y / FACTOR, new_value.y);
        assert_eq!(TEST_Z / FACTOR, new_value.z);
    }

    #[test]
    fn mut_add() {
        let mut uut: Value = Default::default();

        uut.mut_add(&Value {
            x: TEST_X,
            y: TEST_Y,
            z: TEST_Z,
        });

        assert_eq!(TEST_X, uut.x);
        assert_eq!(TEST_Y, uut.y);
        assert_eq!(TEST_Z, uut.z);
    }

    #[test]
    fn mut_sub() {
        let mut uut: Value = Default::default();

        uut.mut_sub(&Value {
            x: TEST_X,
            y: TEST_Y,
            z: TEST_Z,
        });

        assert_eq!(TEST_X * -1.0, uut.x);
        assert_eq!(TEST_Y * -1.0, uut.y);
        assert_eq!(TEST_Z * -1.0, uut.z);
    }

    #[test]
    fn mut_mul() {
        let mut uut: Value = Value {
            x: TEST_X,
            y: TEST_Y,
            z: TEST_Z,
        };

        uut.mut_mul(FACTOR);

        assert_eq!(TEST_X * FACTOR, uut.x);
        assert_eq!(TEST_Y * FACTOR, uut.y);
        assert_eq!(TEST_Z * FACTOR, uut.z);
    }

    #[test]
    fn mut_div() {
        let mut uut: Value = Value {
            x: TEST_X,
            y: TEST_Y,
            z: TEST_Z,
        };

        uut.mut_div(FACTOR);

        assert_eq!(TEST_X / FACTOR, uut.x);
        assert_eq!(TEST_Y / FACTOR, uut.y);
        assert_eq!(TEST_Z / FACTOR, uut.z);
    }
}
