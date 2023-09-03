use rand::Rng;

pub struct Dice<const N: u8>;

impl<const N: u8> Dice<N> {
    pub fn roll() -> u8 {
        rand::thread_rng().gen_range(1..=N)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d6() {
        test_dice_roll::<6>()
    }

    #[test]
    fn d8() {
        test_dice_roll::<8>()
    }

    #[test]
    fn d20() {
        test_dice_roll::<20>()
    }

    fn test_dice_roll<const N: u8>() {
        let result = Dice::<N>::roll();

        assert!(result >= 1, "Expected {result} to be greater than 1");
        assert!(
            result <= N,
            "Expected {result} to be less or equal than {N}"
        );
    }
}
