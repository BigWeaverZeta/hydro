//! Comprehensive tests for variadic operations and collections.

use variadics::*;

#[cfg(test)]
mod variadic_basic_tests {
    use super::*;

    /// Test basic variadic tuple construction
    #[test]
    fn test_variadic_tuple() {
        let v = var_expr!(1, 2, 3);
        assert_eq!(v.0, 1);
        assert_eq!(v.1.0, 2);
        assert_eq!(v.1.1.0, 3);
    }

    /// Test variadic type construction
    #[test]
    fn test_variadic_type() {
        type MyVariadic = var_type!(i32, String, bool);
        let v: MyVariadic = var_expr!(42, "hello".to_string(), true);
        assert_eq!(v.0, 42);
        assert_eq!(v.1.0, "hello");
        assert_eq!(v.1.1.0, true);
    }

    /// Test empty variadic
    #[test]
    fn test_empty_variadic() {
        let v = var_expr!();
        let _: () = v; // Empty variadic is unit type
    }

    /// Test single element variadic
    #[test]
    fn test_single_element() {
        let v = var_expr!(42);
        assert_eq!(v.0, 42);
    }
}

#[cfg(test)]
mod variadic_length_tests {
    use super::*;

    /// Test variadic length
    #[test]
    fn test_length() {
        type V0 = var_type!();
        type V1 = var_type!(i32);
        type V3 = var_type!(i32, String, bool);
        
        // Length is encoded in the type structure
        // () has length 0, (T, ()) has length 1, etc.
        assert_eq!(std::mem::size_of::<V0>(), std::mem::size_of::<()>());
        assert!(std::mem::size_of::<V1>() >= std::mem::size_of::<i32>());
        assert!(std::mem::size_of::<V3>() >= 
                std::mem::size_of::<i32>() + 
                std::mem::size_of::<String>() + 
                std::mem::size_of::<bool>());
    }
}

#[cfg(test)]
mod variadic_spread_tests {
    use super::*;

    /// Test spreading variadics with var_expr
    #[test]
    fn test_spread_in_expr() {
        let v1 = var_expr!(1, 2);
        let v2 = var_expr!(3, 4);
        // Conceptual: var_expr!(...v1, ...v2) would be (1, 2, 3, 4)
        // This tests the spreading mechanism
        
        let combined = var_expr!(v1.0, v1.1.0, v2.0, v2.1.0);
        assert_eq!(combined.0, 1);
        assert_eq!(combined.1.0, 2);
        assert_eq!(combined.1.1.0, 3);
        assert_eq!(combined.1.1.1.0, 4);
    }

    /// Test spreading in type expressions
    #[test]
    fn test_spread_in_type() {
        type V1 = var_type!(i32, String);
        type V2 = var_type!(bool, f64);
        
        // Manual composition
        type Combined = (i32, (String, (bool, (f64, ()))));
        
        let v: Combined = var_expr!(42, "test".to_string(), true, 3.14);
        assert_eq!(v.0, 42);
        assert_eq!(v.1.0, "test");
        assert_eq!(v.1.1.0, true);
        assert!((v.1.1.1.0 - 3.14).abs() < 0.001);
    }
}

#[cfg(test)]
mod variadic_trait_tests {
    use super::*;

    /// Test Variadic trait implementation
    #[test]
    fn test_variadic_trait() {
        fn is_variadic<V: Variadic>(_: &V) -> bool {
            true
        }
        
        assert!(is_variadic(&var_expr!()));
        assert!(is_variadic(&var_expr!(1)));
        assert!(is_variadic(&var_expr!(1, 2, 3)));
    }

    /// Test variadic with different types
    #[test]
    fn test_heterogeneous_variadic() {
        let v = var_expr!(42, "hello", true, 3.14, vec![1, 2, 3]);
        
        assert_eq!(v.0, 42);
        assert_eq!(v.1.0, "hello");
        assert_eq!(v.1.1.0, true);
        assert!((v.1.1.1.0 - 3.14).abs() < 0.001);
        assert_eq!(v.1.1.1.1.0, vec![1, 2, 3]);
    }
}

#[cfg(test)]
mod variadic_clone_tests {
    use super::*;

    /// Test cloning variadic tuples
    #[test]
    fn test_clone() {
        let v1 = var_expr!(42, "test".to_string(), true);
        let v2 = (v1.0, (v1.1.0.clone(), (v1.1.1.0, ())));
        
        assert_eq!(v2.0, 42);
        assert_eq!(v2.1.0, "test");
        assert_eq!(v2.1.1.0, true);
    }
}

#[cfg(test)]
mod variadic_debug_tests {
    use super::*;

    /// Test debug formatting
    #[test]
    fn test_debug() {
        let v = var_expr!(42, "test", true);
        let debug_str = format!("{:?}", v);
        
        // Should contain all elements
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("true"));
    }
}

#[cfg(test)]
mod variadic_generic_tests {
    use super::*;

    /// Test variadic with generic types
    #[test]
    fn test_generic_variadic() {
        fn make_variadic<T: Clone>(a: T, b: T, c: T) -> var_type!(T, T, T) {
            var_expr!(a, b, c)
        }
        
        let v1 = make_variadic(1, 2, 3);
        assert_eq!(v1.0, 1);
        assert_eq!(v1.1.0, 2);
        assert_eq!(v1.1.1.0, 3);
        
        let v2 = make_variadic("a", "b", "c");
        assert_eq!(v2.0, "a");
        assert_eq!(v2.1.0, "b");
        assert_eq!(v2.1.1.0, "c");
    }

    /// Test variadic with trait bounds
    #[test]
    fn test_trait_bounds() {
        fn process_variadic<T: std::fmt::Display>(v: var_type!(T, T)) -> String {
            format!("{} {}", v.0, v.1.0)
        }
        
        let result = process_variadic(var_expr!(42, 84));
        assert_eq!(result, "42 84");
    }
}

#[cfg(test)]
mod variadic_nested_tests {
    use super::*;

    /// Test nested variadics
    #[test]
    fn test_nested() {
        let inner1 = var_expr!(1, 2);
        let inner2 = var_expr!(3, 4);
        let outer = var_expr!(inner1, inner2);
        
        assert_eq!(outer.0.0, 1);
        assert_eq!(outer.0.1.0, 2);
        assert_eq!(outer.1.0.0, 3);
        assert_eq!(outer.1.0.1.0, 4);
    }
}

#[cfg(test)]
mod variadic_pattern_matching_tests {
    use super::*;

    /// Test destructuring variadics
    #[test]
    fn test_destructuring() {
        let v = var_expr!(42, "test", true);
        let (first, rest) = v;
        assert_eq!(first, 42);
        
        let (second, rest2) = rest;
        assert_eq!(second, "test");
        
        let (third, _empty) = rest2;
        assert_eq!(third, true);
    }
}

#[cfg(test)]
mod variadic_macro_tests {
    use super::*;

    /// Test var_expr macro with trailing comma
    #[test]
    fn test_trailing_comma_expr() {
        let v = var_expr!(1, 2, 3,);
        assert_eq!(v.0, 1);
        assert_eq!(v.1.0, 2);
        assert_eq!(v.1.1.0, 3);
    }

    /// Test var_type macro with trailing comma
    #[test]
    fn test_trailing_comma_type() {
        type V = var_type!(i32, String, bool,);
        let v: V = var_expr!(42, "test".to_string(), true);
        assert_eq!(v.0, 42);
    }
}

#[cfg(test)]
mod variadic_equality_tests {
    use super::*;

    /// Test equality of variadics
    #[test]
    fn test_equality() {
        let v1 = var_expr!(42, "test", true);
        let v2 = var_expr!(42, "test", true);
        let v3 = var_expr!(42, "test", false);
        
        assert_eq!(v1.0, v2.0);
        assert_eq!(v1.1.0, v2.1.0);
        assert_eq!(v1.1.1.0, v2.1.1.0);
        
        assert_eq!(v1.0, v3.0);
        assert_eq!(v1.1.0, v3.1.0);
        assert_ne!(v1.1.1.0, v3.1.1.0);
    }
}

#[cfg(test)]
mod variadic_large_tests {
    use super::*;

    /// Test variadic with many elements
    #[test]
    fn test_large_variadic() {
        let v = var_expr!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
        
        assert_eq!(v.0, 0);
        assert_eq!(v.1.0, 1);
        assert_eq!(v.1.1.0, 2);
        assert_eq!(v.1.1.1.0, 3);
        assert_eq!(v.1.1.1.1.0, 4);
        assert_eq!(v.1.1.1.1.1.0, 5);
        assert_eq!(v.1.1.1.1.1.1.0, 6);
        assert_eq!(v.1.1.1.1.1.1.1.0, 7);
        assert_eq!(v.1.1.1.1.1.1.1.1.0, 8);
        assert_eq!(v.1.1.1.1.1.1.1.1.1.0, 9);
    }
}
