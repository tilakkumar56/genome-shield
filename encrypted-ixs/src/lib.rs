use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    pub struct GenomeInput {
        marker1_a: u128,
        marker2_a: u128,
        marker1_b: u128,
        marker2_b: u128,
        count: u8,
    }

    pub struct MatchResult {
        similarity: u128,
        matched: u8,
        compared: u8,
    }

    #[instruction]
    pub fn compare_genomes(input: Enc<Shared, GenomeInput>) -> Enc<Shared, MatchResult> {
        let d = input.to_arcis();
        let mut matched: u8 = 0;
        let mut compared: u8 = 0;

        let v1 = 0 < d.count;
        let m1 = d.marker1_a == d.marker1_b;
        let nz1 = d.marker1_a != 0;
        if v1 { compared = compared + 1; }
        if v1 && m1 && nz1 { matched = matched + 1; }

        let v2 = 1 < d.count;
        let m2 = d.marker2_a == d.marker2_b;
        let nz2 = d.marker2_a != 0;
        if v2 { compared = compared + 1; }
        if v2 && m2 && nz2 { matched = matched + 1; }

        let similarity: u128 = if compared > 0 {
            (matched as u128) * 10000 / (compared as u128)
        } else { 0 };

        let result = MatchResult { similarity, matched, compared };
        input.owner.from_arcis(result)
    }
}
