/// Taken from crate [crc_any](https://docs.rs/crc-any/2.4.0/crc_any/)
static NO_REF_16_1021: [u16; 256] = [
    0u16, 4129u16, 8258u16, 12387u16, 16516u16, 20645u16, 24774u16, 28903u16, 33032u16, 37161u16,
    41290u16, 45419u16, 49548u16, 53677u16, 57806u16, 61935u16, 4657u16, 528u16, 12915u16, 8786u16,
    21173u16, 17044u16, 29431u16, 25302u16, 37689u16, 33560u16, 45947u16, 41818u16, 54205u16,
    50076u16, 62463u16, 58334u16, 9314u16, 13379u16, 1056u16, 5121u16, 25830u16, 29895u16,
    17572u16, 21637u16, 42346u16, 46411u16, 34088u16, 38153u16, 58862u16, 62927u16, 50604u16,
    54669u16, 13907u16, 9842u16, 5649u16, 1584u16, 30423u16, 26358u16, 22165u16, 18100u16,
    46939u16, 42874u16, 38681u16, 34616u16, 63455u16, 59390u16, 55197u16, 51132u16, 18628u16,
    22757u16, 26758u16, 30887u16, 2112u16, 6241u16, 10242u16, 14371u16, 51660u16, 55789u16,
    59790u16, 63919u16, 35144u16, 39273u16, 43274u16, 47403u16, 23285u16, 19156u16, 31415u16,
    27286u16, 6769u16, 2640u16, 14899u16, 10770u16, 56317u16, 52188u16, 64447u16, 60318u16,
    39801u16, 35672u16, 47931u16, 43802u16, 27814u16, 31879u16, 19684u16, 23749u16, 11298u16,
    15363u16, 3168u16, 7233u16, 60846u16, 64911u16, 52716u16, 56781u16, 44330u16, 48395u16,
    36200u16, 40265u16, 32407u16, 28342u16, 24277u16, 20212u16, 15891u16, 11826u16, 7761u16,
    3696u16, 65439u16, 61374u16, 57309u16, 53244u16, 48923u16, 44858u16, 40793u16, 36728u16,
    37256u16, 33193u16, 45514u16, 41451u16, 53516u16, 49453u16, 61774u16, 57711u16, 4224u16,
    161u16, 12482u16, 8419u16, 20484u16, 16421u16, 28742u16, 24679u16, 33721u16, 37784u16,
    41979u16, 46042u16, 49981u16, 54044u16, 58239u16, 62302u16, 689u16, 4752u16, 8947u16, 13010u16,
    16949u16, 21012u16, 25207u16, 29270u16, 46570u16, 42443u16, 38312u16, 34185u16, 62830u16,
    58703u16, 54572u16, 50445u16, 13538u16, 9411u16, 5280u16, 1153u16, 29798u16, 25671u16,
    21540u16, 17413u16, 42971u16, 47098u16, 34713u16, 38840u16, 59231u16, 63358u16, 50973u16,
    55100u16, 9939u16, 14066u16, 1681u16, 5808u16, 26199u16, 30326u16, 17941u16, 22068u16,
    55628u16, 51565u16, 63758u16, 59695u16, 39368u16, 35305u16, 47498u16, 43435u16, 22596u16,
    18533u16, 30726u16, 26663u16, 6336u16, 2273u16, 14466u16, 10403u16, 52093u16, 56156u16,
    60223u16, 64286u16, 35833u16, 39896u16, 43963u16, 48026u16, 19061u16, 23124u16, 27191u16,
    31254u16, 2801u16, 6864u16, 10931u16, 14994u16, 64814u16, 60687u16, 56684u16, 52557u16,
    48554u16, 44427u16, 40424u16, 36297u16, 31782u16, 27655u16, 23652u16, 19525u16, 15522u16,
    11395u16, 7392u16, 3265u16, 61215u16, 65342u16, 53085u16, 57212u16, 44955u16, 49082u16,
    36825u16, 40952u16, 28183u16, 32310u16, 20053u16, 24180u16, 11923u16, 16050u16, 3793u16,
    7920u16,
];

/// calculate a crc16
///
/// ```ignore
/// |Check |Poly  |Init  |Ref  |XorOut|
/// |---   |---   |---   |---  |---   |
/// |0x29B1|0x1021|0xFFFF|false|0x0000|
/// ```
/// taken from crate [crc_any](https://docs.rs/crc-any/2.4.0/crc_any/) and minimized to our needs
#[derive(Debug)]
pub struct Crc16(u16);

impl Crc16 {
    /// Initializes the crc as 0xFFFF.
    pub fn init() -> Self {
        Self(0xFFFF)
    }

    const BITS: u16 = 16;

    /// Process the current crc sum further with the supplied data.
    pub fn digest<T: ?Sized + AsRef<[u8]>>(&mut self, data: &T) {
        for n in data.as_ref().iter().copied() {
            let index = ((self.0 >> u16::from(Self::BITS - 8)) as u8 ^ n) as usize;
            self.0 = (self.0 << 8) ^ NO_REF_16_1021[index];
        }
    }

    /// Retrieve the current crc sum.
    pub fn get_crc(&self) -> u16 {
        let final_xor = 0x0000;

        let sum = (self.0 ^ final_xor) & Crc16::mask();

        sum
    }

    const fn mask() -> u16 {
        let high_bit = 1 << (Self::BITS - 1);
        let mask = ((high_bit - 1) << 1) | 1;
        mask
    }
}

#[cfg(test)]
mod test {
    use ::test::Bencher;

    use super::*;
    #[test]
    fn calculate_crc() {
        let payload = "123456789";

        let mut ref_impl = crc_any::CRCu16::crc16ccitt_false();
        ref_impl.digest(payload);
        let ref_crc = ref_impl.get_crc();

        let mut crc_impl = Crc16::init();
        crc_impl.digest(payload);
        let crc = crc_impl.get_crc();

        assert_eq!(ref_crc, crc);
    }

    #[test]
    fn calculate_crc_process_crc_further() {
        let payload = "123456789";

        let mut ref_impl = crc_any::CRCu16::crc16ccitt_false();
        ref_impl.digest(payload);
        ref_impl.digest(payload);
        let ref_crc = ref_impl.get_crc();

        let mut crc_impl = Crc16::init();
        crc_impl.digest(payload);
        crc_impl.digest(payload);
        let crc = crc_impl.get_crc();

        assert_eq!(ref_crc, crc);
    }

    #[bench]
    fn bench_crc_data_len_9(b: &mut Bencher) {
        let payload = "123456789";
        b.iter(|| {
            let mut crc_impl = Crc16::init();
            crc_impl.digest(payload);
            let crc = crc_impl.get_crc();
            crc
        });
    }
}
