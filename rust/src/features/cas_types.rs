// Copyright 2025 John Doe
// SPDX-License-Identifier: MIT OR Apache-2.0

// This file is part of the Advanced CAS implementation for kistaverk.
// It defines a flexible Number enum that currently wraps f64 but is designed
// for future extension to support arbitrary-precision types.

use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};

/// A flexible numeric type that can represent different precision levels.
/// Currently only supports Fast(f64), but designed for future extension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    /// Fast floating-point representation using f64
    Fast(f64),
}

impl Number {
    /// Create a Number from an f64 value
    pub fn from_f64(value: f64) -> Self {
        Number::Fast(value)
    }
    
    /// Convert Number to f64 (currently just unwraps the Fast variant)
    pub fn to_f64(self) -> f64 {
        match self {
            Number::Fast(value) => value,
        }
    }
}

// Implement arithmetic operations for Number
impl Add for Number {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fast(a), Number::Fast(b)) => Number::Fast(a + b),
        }
    }
}

impl Sub for Number {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fast(a), Number::Fast(b)) => Number::Fast(a - b),
        }
    }
}

impl Mul for Number {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fast(a), Number::Fast(b)) => Number::Fast(a * b),
        }
    }
}

impl Div for Number {
    type Output = Self;
    
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fast(a), Number::Fast(b)) => Number::Fast(a / b),
        }
    }
}

// Implement Neg operation
impl std::ops::Neg for Number {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        match self {
            Number::Fast(a) => Number::Fast(-a),
        }
    }
}

// Implement comparison operations
impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Number::Fast(a), Number::Fast(b)) => a.partial_cmp(b),
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
        let sum = a + b;
        assert_eq!(sum.to_f64(), 15.0);
        
        // Test subtraction
        let diff = a - b;
        assert_eq!(diff.to_f64(), 5.0);
        
        // Test multiplication
        let product = a * b;
        assert_eq!(product.to_f64(), 50.0);
        
        // Test division
        let quotient = a / b;
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
}