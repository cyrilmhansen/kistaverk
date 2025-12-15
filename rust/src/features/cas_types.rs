// Copyright 2025 John Doe
// SPDX-License-Identifier: MIT OR Apache-2.0

// This file is part of the Advanced CAS implementation for kistaverk.
// It defines a flexible Number enum that currently wraps f64 but is designed
// for future extension to support arbitrary-precision types.

use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};

/// A flexible numeric type that can represent different precision levels.
/// Currently supports Fast(f64), with optional Precise(rug::Float) for arbitrary precision.
#[derive(Debug, PartialEq)]
pub enum Number {
    /// Fast floating-point representation using f64
    Fast(f64),
    /// Arbitrary precision representation using rug::Float (available with "precision" feature)
    #[cfg(feature = "precision")]
    Precise(rug::Float),
}

impl Clone for Number {
    fn clone(&self) -> Self {
        match self {
            // Fast variant: f64 is Copy, so this is essentially a no-op
            Number::Fast(value) => Number::Fast(*value),
            #[cfg(feature = "precision")]
            // Precise variant: rug::Float needs actual cloning
            Number::Precise(value) => Number::Precise(value.clone()),
        }
    }
}

// Implement Copy trait when precision feature is disabled
#[cfg(not(feature = "precision"))]
impl Copy for Number {}

impl Number {
    /// Create a Number from an f64 value
    pub fn from_f64(value: f64) -> Self {
        Number::Fast(value)
    }
    
    /// Create a Number from a rug::Float (available with "precision" feature)
    #[cfg(feature = "precision")]
    pub fn from_rug_float(value: rug::Float) -> Self {
        Number::Precise(value)
    }
    
    /// Convert Number to f64
    pub fn to_f64(self) -> f64 {
        match self {
            Number::Fast(value) => value,
            #[cfg(feature = "precision")]
            Number::Precise(value) => value.to_f64(),
        }
    }
    
    /// Convert Number to rug::Float (available with "precision" feature)
    #[cfg(feature = "precision")]
    pub fn to_rug_float(self) -> rug::Float {
        match self {
            Number::Fast(value) => rug::Float::with_val(53, value),
            Number::Precise(value) => value,
        }
    }
    
    /// Convert to Fast variant (losing precision if necessary)
    #[allow(dead_code)]
    pub fn to_fast(self) -> Self {
        Number::Fast(self.to_f64())
    }
    
    /// Convert to Precise variant (available with "precision" feature)
    #[cfg(feature = "precision")]
    pub fn to_precise(self) -> Self {
        Number::Precise(self.to_rug_float())
    }
}

// Implement arithmetic operations for Number
impl Add for Number {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fast(a), Number::Fast(b)) => Number::Fast(a + b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Precise(b)) => Number::Precise(a + b),
            #[cfg(feature = "precision")]
            (Number::Fast(a), Number::Precise(b)) => Number::Precise(rug::Float::with_val(53, a) + b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Fast(b)) => Number::Precise(a + rug::Float::with_val(53, b)),
        }
    }
}

impl Sub for Number {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fast(a), Number::Fast(b)) => Number::Fast(a - b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Precise(b)) => Number::Precise(a - b),
            #[cfg(feature = "precision")]
            (Number::Fast(a), Number::Precise(b)) => Number::Precise(rug::Float::with_val(53, a) - b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Fast(b)) => Number::Precise(a - rug::Float::with_val(53, b)),
        }
    }
}

impl Mul for Number {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fast(a), Number::Fast(b)) => Number::Fast(a * b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Precise(b)) => Number::Precise(a * b),
            #[cfg(feature = "precision")]
            (Number::Fast(a), Number::Precise(b)) => Number::Precise(rug::Float::with_val(53, a) * b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Fast(b)) => Number::Precise(a * rug::Float::with_val(53, b)),
        }
    }
}

impl Div for Number {
    type Output = Self;
    
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fast(a), Number::Fast(b)) => Number::Fast(a / b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Precise(b)) => Number::Precise(a / b),
            #[cfg(feature = "precision")]
            (Number::Fast(a), Number::Precise(b)) => Number::Precise(rug::Float::with_val(53, a) / b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Fast(b)) => Number::Precise(a / rug::Float::with_val(53, b)),
        }
    }
}

// Implement Neg operation
impl std::ops::Neg for Number {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        match self {
            Number::Fast(a) => Number::Fast(-a),
            #[cfg(feature = "precision")]
            Number::Precise(a) => Number::Precise(-a),
        }
    }
}

// Implement is_finite method for Number
impl Number {
    /// Check if the number is finite (not NaN or infinity)
    pub fn is_finite(&self) -> bool {
        match self {
            Number::Fast(value) => value.is_finite(),
            #[cfg(feature = "precision")]
            Number::Precise(value) => value.is_finite(),
        }
    }
}

// Implement comparison operations
impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Number::Fast(a), Number::Fast(b)) => a.partial_cmp(b),
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Precise(b)) => a.partial_cmp(b),
            #[cfg(feature = "precision")]
            (Number::Fast(a), Number::Precise(b)) => {
                let a_precise = rug::Float::with_val(53, *a);
                a_precise.partial_cmp(b)
            }
            #[cfg(feature = "precision")]
            (Number::Precise(a), Number::Fast(b)) => {
                let b_precise = rug::Float::with_val(53, *b);
                a.partial_cmp(&b_precise)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_number_creation() {
        let num = Number::from_f64(42.5);
        assert_eq!(num.to_f64(), 42.5);
    }

    #[test]
    fn test_arithmetic_operations() {
        let a = Number::from_f64(10.0);
        let b = Number::from_f64(5.0);
        
        // Test addition
        let sum = a.clone() + b.clone();
        assert_eq!(sum.to_f64(), 15.0);
        
        // Test subtraction
        let diff = a.clone() - b.clone();
        assert_eq!(diff.to_f64(), 5.0);
        
        // Test multiplication
        let product = a.clone() * b.clone();
        assert_eq!(product.to_f64(), 50.0);
        
        // Test division
        let quotient = a.clone() / b.clone();
        assert_eq!(quotient.to_f64(), 2.0);
    }

    #[test]
    fn test_comparison_operations() {
        let a = Number::from_f64(10.0);
        let b = Number::from_f64(5.0);
        let c = Number::from_f64(10.0);
        
        // Test equality
        assert_eq!(a, c);
        assert_ne!(a, b);
        
        // Test ordering
        assert!(a > b);
        assert!(b < a);
        assert!(a >= c);
        assert!(b <= a);
    }

    #[test]
    fn test_complex_arithmetic() {
        let a = Number::from_f64(PI);
        let b = Number::from_f64(2.0);
        
        // Test more complex operations
        let result = (a * b) + Number::from_f64(1.0);
        let expected = (PI * 2.0) + 1.0;
        assert!((result.to_f64() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_chained_operations() {
        let a = Number::from_f64(10.0);
        let b = Number::from_f64(2.0);
        let c = Number::from_f64(3.0);
        
        // Test chained operations: (10 + 2) * 3 - 5 = 31
        let result = ((a + b) * c) - Number::from_f64(5.0);
        assert_eq!(result.to_f64(), 31.0);
    }

    #[cfg(feature = "precision")]
    #[test]
    fn test_precise_arithmetic() {
        // Test precise number creation
        let pi_precise = rug::Float::with_val(100, PI);
        let num = Number::from_rug_float(pi_precise.clone());
        
        // Test conversion to rug::Float
        let converted = num.to_rug_float();
        assert_eq!(converted, pi_precise);
        
        // Test precise arithmetic
        let a = Number::from_rug_float(rug::Float::with_val(100, 10.0));
        let b = Number::from_rug_float(rug::Float::with_val(100, 3.0));
        
        let sum = a.clone() + b.clone();
        assert_eq!(sum.to_rug_float(), rug::Float::with_val(100, 13.0));
        
        let diff = a.clone() - b.clone();
        assert_eq!(diff.to_rug_float(), rug::Float::with_val(100, 7.0));
        
        let product = a.clone() * b.clone();
        assert_eq!(product.to_rug_float(), rug::Float::with_val(100, 30.0));
        
        let quotient = a.clone() / b.clone();
        let expected_div = rug::Float::with_val(100, 10.0) / rug::Float::with_val(100, 3.0);
        assert_eq!(quotient.to_rug_float(), expected_div);
    }

    #[cfg(feature = "precision")]
    #[test]
    fn test_mixed_precision_arithmetic() {
        let fast_num = Number::from_f64(10.0);
        let precise_num = Number::from_rug_float(rug::Float::with_val(100, 3.0));
        
        // Test mixed addition
        let sum = fast_num.clone() + precise_num.clone();
        assert_eq!(sum.to_rug_float(), rug::Float::with_val(100, 13.0));
        
        // Test mixed subtraction
        let diff = precise_num.clone() - fast_num.clone();
        let expected_diff = rug::Float::with_val(100, 3.0) - rug::Float::with_val(100, 10.0);
        assert_eq!(diff.to_rug_float(), expected_diff);
    }

    #[cfg(feature = "precision")]
    #[test]
    fn test_precision_conversion() {
        // Test conversion from fast to precise
        let fast_num = Number::from_f64(3.141592653589793);
        let precise_num = fast_num.clone().to_precise();
        
        // Test conversion back to fast
        let back_to_fast = precise_num.to_fast();
        
        // Should be approximately equal (within f64 precision)
        assert!((back_to_fast.to_f64() - fast_num.to_f64()).abs() < f64::EPSILON);
    }
}