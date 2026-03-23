use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    pub struct VoteInput {
        option_idx: u8,
        weight: u128,
        num_options: u8,
    }

    pub struct VoteResult {
        is_valid: u8,
        option_idx: u8,
        weight: u128,
    }

    #[instruction]
    pub fn cast_vote(vote: Enc<Shared, VoteInput>) -> Enc<Shared, VoteResult> {
        let v = vote.to_arcis();
        let valid = v.option_idx < v.num_options;
        let is_valid: u8 = if valid { 1 } else { 0 };
        let result = VoteResult {
            is_valid,
            option_idx: v.option_idx,
            weight: v.weight,
        };
        vote.owner.from_arcis(result)
    }
}
