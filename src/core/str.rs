pub(crate) fn to_lower_en_kebab_case(s: &str) -> String {
    convert_tr_chars_to_en(&to_kebab_case(&s.to_lowercase()))
}

pub(crate) fn to_kebab_case(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}

pub(crate) fn convert_tr_chars_to_en(s: &str) -> String {
       let tr_to_en = [
        ('ç', 'c'), ('Ç', 'C'),
        ('ğ', 'g'), ('Ğ', 'G'),
        ('ş', 's'), ('Ş', 'S'),
        ('ü', 'u'), ('Ü', 'U'),
        ('ö', 'o'), ('Ö', 'O'),
        ('ı', 'i'), ('İ', 'I'),
    ];

    // Çok performanslı bir metot olmadı, O(n^2), daha iyi bir algoritma arayabiliriz
    s.chars().map(|c| {
        // (ç, _) varsa atıyorum, (_, c) alacağız ve new = c olacak, yani karakterin karşılığını almış olacağız
        if let Some(&(_, new)) = tr_to_en.iter().find(|&&(t, _)| t == c) {
            new
        } else {
            c
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_lower_en_kebab_case() {
        // Basit Türkçe karakterli test
        assert_eq!(to_lower_en_kebab_case("Çay Kahve"), "cay-kahve");
        assert_eq!(to_lower_en_kebab_case("Şeker Çiğdem"), "seker-cigdem");

        // Büyük harf ve boşluk testi
        assert_eq!(to_lower_en_kebab_case("BÜYÜK HARF"), "buyuk-harf");

        // Özel karakter ve boşluk testi
        assert_eq!(to_lower_en_kebab_case("Test @ Çözüm!"), "test-@-cozum!");

        // Boş string testi
        assert_eq!(to_lower_en_kebab_case(""), "");

        // Tek kelime testi
        assert_eq!(to_lower_en_kebab_case("Gökyüzü"), "gokyuzu");

        // Ardışık boşluk testi
        assert_eq!(to_lower_en_kebab_case("Çay   Kahve"), "cay-kahve");
    }

    #[test]
    fn test_to_kebab_case() {
        // Basit boşluk birleştirme testi
        assert_eq!(to_kebab_case("Çay Kahve"), "çay-kahve");

        // Büyük harf testi
        assert_eq!(to_kebab_case("BÜYÜK HARF"), "büyük-harf");

        // Özel karakter testi
        assert_eq!(to_kebab_case("Test @ Çözüm!"), "test-@-çözüm!");

        // Boş string testi
        assert_eq!(to_kebab_case(""), "");

        // Tek kelime testi
        assert_eq!(to_kebab_case("Gökyüzü"), "gökyüzü");

        // Ardışık boşluk testi
        assert_eq!(to_kebab_case("Çay   Kahve"), "çay-kahve");
    }

    #[test]
    fn test_convert_tr_chars_to_en() {
        // Tüm Türkçe karakterler testi
        assert_eq!(convert_tr_chars_to_en("ÇŞÜĞÖİ"), "CSUGOI");
        assert_eq!(convert_tr_chars_to_en("çşüğöı"), "csugoi");
        assert_eq!(convert_tr_chars_to_en("ÇçŞşÜüĞğÖöİı"), "CcSsUuGgOoIi");

        // Karışık Türkçe ve İngilizce karakter testi
        assert_eq!(convert_tr_chars_to_en("Merhaba Çay"), "Merhaba Cay");

        // Boş string testi
        assert_eq!(convert_tr_chars_to_en(""), "");

        // Sadece İngilizce karakter testi
        assert_eq!(convert_tr_chars_to_en("Hello World"), "Hello World");

        // Özel karakter testi
        assert_eq!(convert_tr_chars_to_en("Test@Çözüm!"), "Test@Cozum!");
    }
}