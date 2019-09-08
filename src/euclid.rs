pub fn modulo(a: i32, b: i32) -> i32
{
    let m = a % b;
    if m < 0 {
        if b < 0 {
            m - b
        } else {
            m + b
        }
    } else {
        m
    }
}

pub fn euclid(a: i32, b: i32) -> i32
{
    if b == 0 {
        a
    } else {
        euclid(b, modulo(a, b))
    }
}

#[cfg(test)]
pub mod tests {
    use super::euclid;

    #[test]
    pub fn test_euclid() {
        assert_eq!(euclid(5, 7), 1);
        assert_eq!(euclid(12, 4), 4);
        assert_eq!(euclid(18, 60), 6);
        assert_eq!(euclid(8, 8), 8);
        assert_eq!(euclid(0, 17), 17);
        assert_eq!(euclid(-4, 8), 4);
        assert_eq!(euclid(-2, 1), 1);
        assert_eq!(euclid(1, 0), 1);
    }
}