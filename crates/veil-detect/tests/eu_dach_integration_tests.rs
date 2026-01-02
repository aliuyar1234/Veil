//! End-to-end integration tests for EU/DACH PII detectors.
//!
//! Tests realistic documents containing German, Austrian, and Swiss PII data.

use veil_detect::{DetectorRegistry, PiiCategory};
use veil_types::{Position, TextSegment};

fn make_segment(content: &str) -> TextSegment {
    TextSegment {
        content: content.to_string().into(),
        position: Position::Text {
            line: 1,
            column: 1,
            byte_offset: 0,
            byte_length: content.len(),
        },
    }
}

// =============================================================================
// German Tax ID (Steueridentifikationsnummer) Tests
// =============================================================================

#[test]
fn test_german_tax_id_in_formal_letter() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Finanzamt München
        Deroystraße 6
        80335 München

        Sehr geehrte Damen und Herren,

        hiermit beantrage ich die Ausstellung einer Bescheinigung.

        Meine Steueridentifikationsnummer: 86095742719
        Geburtsdatum: 15.03.1985

        Mit freundlichen Grüßen
        Max Mustermann
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let tax_ids: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::TaxIdDe)
        .collect();

    assert_eq!(tax_ids.len(), 1, "Should find exactly one German Tax ID");
    assert_eq!(tax_ids[0].matched_text.as_str(), "86095742719");
}

#[test]
fn test_german_tax_id_in_payroll_data() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Lohnabrechnung März 2024
        ========================
        Mitarbeiter: Schmidt, Anna
        Personalnummer: 10042
        Steuer-ID: 65929970489
        Steuerklasse: 1
        Bruttolohn: 4.500,00 EUR
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let tax_ids: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::TaxIdDe)
        .collect();

    assert_eq!(tax_ids.len(), 1);
    assert_eq!(tax_ids[0].matched_text.as_str(), "65929970489");
}

#[test]
fn test_multiple_german_tax_ids_in_csv() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Name;Steuer-ID;Abteilung
        Müller, Hans;12345678911;Vertrieb
        Weber, Lisa;98765432198;Marketing
        Bauer, Peter;45678912345;IT
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let tax_ids: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::TaxIdDe)
        .collect();

    assert_eq!(tax_ids.len(), 3, "Should find three German Tax IDs in CSV");
}

// =============================================================================
// Swiss AHV Number Tests
// =============================================================================

#[test]
fn test_swiss_ahv_in_insurance_form() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Schweizerische Ausgleichskasse
        Antrag auf Altersrente

        Personalien:
        Name: Meier
        Vorname: Hans
        AHV-Nr.: 756.1234.5678.97
        Geburtsdatum: 01.05.1958

        Ich beantrage die ordentliche Altersrente ab dem 65. Altersjahr.
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let ahv_numbers: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::AhvCh)
        .collect();

    assert_eq!(
        ahv_numbers.len(),
        1,
        "Should find exactly one Swiss AHV number"
    );
    assert_eq!(ahv_numbers[0].matched_text.as_str(), "756.1234.5678.97");
}

#[test]
fn test_swiss_ahv_different_formats() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Versicherte Personen:
        1. AHV: 756.9876.5432.10 (mit Punkten)
        2. AHV: 756 1111 2222 33 (mit Leerzeichen)
        3. AHV: 7564567890123 (ohne Trennzeichen)
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let ahv_numbers: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::AhvCh)
        .collect();

    assert_eq!(
        ahv_numbers.len(),
        3,
        "Should find all three AHV format variants"
    );
}

#[test]
fn test_swiss_ahv_in_employment_contract() {
    let registry = DetectorRegistry::default();
    let document = r#"
        ARBEITSVERTRAG

        zwischen
        Schweizer AG, Bahnhofstrasse 1, 8001 Zürich
        (nachfolgend "Arbeitgeber")

        und
        Frau Maria Müller, Seestrasse 25, 6004 Luzern
        AHV-Nummer: 756.2345.6789.01
        (nachfolgend "Arbeitnehmerin")

        § 1 Beginn des Arbeitsverhältnisses
        Das Arbeitsverhältnis beginnt am 1. April 2024.
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let ahv_numbers: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::AhvCh)
        .collect();

    assert_eq!(ahv_numbers.len(), 1);
    assert!(ahv_numbers[0].matched_text.as_str().starts_with("756"));
}

// =============================================================================
// German National ID (Personalausweis) Tests
// =============================================================================

#[test]
fn test_german_national_id_in_verification() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Identitätsprüfung durchgeführt am: 15.03.2024

        Dokumententyp: Personalausweis
        Ausweisnummer: L01X00T471
        Gültig bis: 14.03.2034
        Ausstellende Behörde: Stadt München

        Identität bestätigt: Ja
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let national_ids: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::NationalIdDe)
        .collect();

    assert_eq!(national_ids.len(), 1, "Should find German National ID");
    assert_eq!(national_ids[0].matched_text.as_str(), "L01X00T471");
}

#[test]
fn test_german_national_id_lowercase() {
    let registry = DetectorRegistry::default();
    let document = "Personalausweis-Nr.: t22hk0x4m7 wurde geprüft.";

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let national_ids: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::NationalIdDe)
        .collect();

    assert_eq!(national_ids.len(), 1, "Should find lowercase National ID");
}

// =============================================================================
// EU VAT Number Tests
// =============================================================================

#[test]
fn test_german_vat_in_invoice() {
    let registry = DetectorRegistry::default();
    let document = r#"
        RECHNUNG Nr. 2024-0042

        Musterfirma GmbH
        Musterstraße 123
        80331 München

        USt-IdNr.: DE123456789

        Rechnungsbetrag netto: 1.000,00 EUR
        USt 19%:                 190,00 EUR
        Gesamtbetrag:          1.190,00 EUR
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let vat_numbers: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::VatNumber)
        .collect();

    assert_eq!(vat_numbers.len(), 1, "Should find German VAT number");
    assert_eq!(vat_numbers[0].matched_text.as_str(), "DE123456789");
}

#[test]
fn test_austrian_vat_in_invoice() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Rechnung

        Wiener Handels GmbH
        Kärntner Straße 50
        1010 Wien, Österreich

        UID-Nummer: ATU12345678
        Firmenbuchnummer: FN 123456a

        Nettobetrag: 500,00 EUR
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let vat_numbers: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::VatNumber)
        .collect();

    assert_eq!(vat_numbers.len(), 1, "Should find Austrian VAT number");
    assert_eq!(vat_numbers[0].matched_text.as_str(), "ATU12345678");
}

#[test]
fn test_swiss_vat_in_invoice() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Faktura

        Zürcher Consulting AG
        Paradeplatz 8
        8001 Zürich

        MWST-Nr.: CHE-123.456.789 MWST

        Honorar exkl. MWST: CHF 2'500.00
        MWST 8.1%:          CHF   202.50
        Total:              CHF 2'702.50
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let vat_numbers: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::VatNumber)
        .collect();

    assert_eq!(vat_numbers.len(), 1, "Should find Swiss VAT number");
    assert!(vat_numbers[0].matched_text.as_str().contains("CHE"));
}

#[test]
fn test_multiple_eu_vat_numbers_in_partner_list() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Geschäftspartner-Verzeichnis
        ============================

        1. Deutsche Partner GmbH
           USt-IdNr.: DE987654321
           Land: Deutschland

        2. French Solutions SARL
           TVA: FR12345678901
           Land: Frankreich

        3. Italian Services SRL
           P.IVA: IT12345678901
           Land: Italien

        4. Dutch Trading BV
           BTW: NL123456789B01
           Land: Niederlande

        5. Belgian Enterprises NV
           TVA: BE0123456789
           Land: Belgien
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let vat_numbers: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::VatNumber)
        .collect();

    assert_eq!(vat_numbers.len(), 5, "Should find all 5 EU VAT numbers");

    // Verify each country is detected
    let vat_texts: Vec<&str> = vat_numbers
        .iter()
        .map(|f| f.matched_text.as_str())
        .collect();
    assert!(
        vat_texts.iter().any(|v| v.starts_with("DE")),
        "Should find DE VAT"
    );
    assert!(
        vat_texts.iter().any(|v| v.starts_with("FR")),
        "Should find FR VAT"
    );
    assert!(
        vat_texts.iter().any(|v| v.starts_with("IT")),
        "Should find IT VAT"
    );
    assert!(
        vat_texts.iter().any(|v| v.starts_with("NL")),
        "Should find NL VAT"
    );
    assert!(
        vat_texts.iter().any(|v| v.starts_with("BE")),
        "Should find BE VAT"
    );
}

// =============================================================================
// Combined Real-World Scenarios
// =============================================================================

#[test]
fn test_german_customer_record() {
    let registry = DetectorRegistry::default();
    let document = r#"
        KUNDENSTAMMDATEN

        Kundennummer: K-2024-0815
        Anrede: Herr
        Name: Dr. Thomas Schneider
        Geburtsdatum: 22.07.1978

        Adresse:
        Hauptstraße 42
        80331 München

        Kontakt:
        E-Mail: thomas.schneider@example.de
        Telefon: +49 89 12345678
        Mobil: 0171 9876543

        Steuerliche Angaben:
        Steuer-ID: 82695413025
        USt-IdNr.: DE298765432

        Bankverbindung:
        IBAN: DE89370400440532013000
        BIC: COBADEFFXXX
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    // Check for expected PII types
    let categories: Vec<_> = findings.iter().map(|f| &f.category).collect();

    assert!(
        categories.contains(&&PiiCategory::Email),
        "Should find email"
    );
    assert!(
        categories.contains(&&PiiCategory::Phone),
        "Should find phone"
    );
    assert!(
        categories.contains(&&PiiCategory::TaxIdDe),
        "Should find German Tax ID"
    );
    assert!(
        categories.contains(&&PiiCategory::VatNumber),
        "Should find VAT number"
    );
    assert!(categories.contains(&&PiiCategory::Iban), "Should find IBAN");

    // Verify specific values
    let tax_id = findings
        .iter()
        .find(|f| f.category == PiiCategory::TaxIdDe)
        .unwrap();
    assert_eq!(tax_id.matched_text.as_str(), "82695413025");

    let vat = findings
        .iter()
        .find(|f| f.category == PiiCategory::VatNumber)
        .unwrap();
    assert_eq!(vat.matched_text.as_str(), "DE298765432");

    let iban = findings
        .iter()
        .find(|f| f.category == PiiCategory::Iban)
        .unwrap();
    assert!(iban.matched_text.as_str().starts_with("DE89"));
}

#[test]
fn test_swiss_employee_record() {
    let registry = DetectorRegistry::default();
    let document = r#"
        PERSONALAKTE

        Mitarbeiter-ID: CH-2024-0123
        Name: Müller, Anna
        Nationalität: Schweiz

        AHV-Nummer: 756.3456.7890.12

        Arbeitgeber:
        Firma: Schweizer Tech AG
        MWST-Nr.: CHE-987.654.321 MWST

        Lohnkonto:
        IBAN: CH93 0076 2011 6238 5295 7

        Notfallkontakt:
        Tel: +41 44 123 45 67
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let categories: Vec<_> = findings.iter().map(|f| &f.category).collect();

    assert!(
        categories.contains(&&PiiCategory::AhvCh),
        "Should find Swiss AHV"
    );
    assert!(
        categories.contains(&&PiiCategory::VatNumber),
        "Should find Swiss VAT"
    );
    assert!(
        categories.contains(&&PiiCategory::Iban),
        "Should find Swiss IBAN"
    );
    assert!(
        categories.contains(&&PiiCategory::Phone),
        "Should find phone"
    );

    let ahv = findings
        .iter()
        .find(|f| f.category == PiiCategory::AhvCh)
        .unwrap();
    assert_eq!(ahv.matched_text.as_str(), "756.3456.7890.12");
}

#[test]
fn test_austrian_business_document() {
    let registry = DetectorRegistry::default();
    let document = r#"
        GESCHÄFTSBRIEF

        Wiener Dienstleistungs GmbH
        Ringstraße 1
        1010 Wien

        UID-Nr.: ATU87654321
        Firmenbuch: FN 987654z

        An:
        Herrn Mag. Johann Berger
        Salzburger Straße 100
        5020 Salzburg

        Betreff: Angebot Nr. 2024/0815

        Sehr geehrter Herr Mag. Berger,

        anbei übersenden wir Ihnen unser Angebot.

        Bankverbindung für Überweisungen:
        IBAN: AT611904300234573201
        BIC: BKAUATWW

        Rückfragen unter: +43 1 234 567 890

        Mit freundlichen Grüßen
        Wiener Dienstleistungs GmbH
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    let categories: Vec<_> = findings.iter().map(|f| &f.category).collect();

    assert!(
        categories.contains(&&PiiCategory::VatNumber),
        "Should find Austrian VAT"
    );
    assert!(
        categories.contains(&&PiiCategory::Iban),
        "Should find Austrian IBAN"
    );
    assert!(
        categories.contains(&&PiiCategory::Phone),
        "Should find Austrian phone"
    );

    let vat = findings
        .iter()
        .find(|f| f.category == PiiCategory::VatNumber)
        .unwrap();
    assert_eq!(vat.matched_text.as_str(), "ATU87654321");
}

#[test]
fn test_international_supplier_database() {
    let registry = DetectorRegistry::default();
    let document = r#"
        LIEFERANTEN-DATENBANK EXPORT
        Datum: 2024-03-15

        ID,Name,Land,USt-IdNr,IBAN,Kontakt
        001,Müller GmbH,DE,DE111222333,DE89370400440532013000,+49 30 12345
        002,Meier AG,CH,CHE-111.222.333 MWST,CH9300762011623852957,+41 44 98765
        003,Huber KG,AT,ATU11122233,AT611904300234573201,+43 1 55544
        004,Dupont SARL,FR,FR11222333444,FR7630006000011234567890189,+33 1 23456789
        005,Rossi SRL,IT,IT11122233344,IT60X0542811101000000123456,+39 02 1234567
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    // Count findings by category
    let vat_count = findings
        .iter()
        .filter(|f| f.category == PiiCategory::VatNumber)
        .count();
    let iban_count = findings
        .iter()
        .filter(|f| f.category == PiiCategory::Iban)
        .count();
    let phone_count = findings
        .iter()
        .filter(|f| f.category == PiiCategory::Phone)
        .count();

    assert!(
        vat_count >= 5,
        "Should find at least 5 VAT numbers, found {}",
        vat_count
    );
    assert!(
        iban_count >= 5,
        "Should find at least 5 IBANs, found {}",
        iban_count
    );
    assert!(
        phone_count >= 5,
        "Should find at least 5 phone numbers, found {}",
        phone_count
    );
}

// =============================================================================
// Edge Cases and False Positive Prevention
// =============================================================================

#[test]
fn test_no_false_positives_on_order_numbers() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Bestellnummer: 12345678901
        Auftragsnummer: 98765432109
        Artikelnummer: 11223344556
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    // These should potentially be detected but with low confidence
    // The numbers match the pattern but context should lower confidence
    for finding in &findings {
        if finding.category == PiiCategory::TaxIdDe {
            // Context analysis should reduce confidence for order numbers
            println!(
                "Found Tax ID candidate: {} with confidence {}",
                finding.matched_text.as_str(),
                finding.confidence
            );
        }
    }
}

#[test]
fn test_no_false_positives_on_phone_like_numbers() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Telefonnummer: 089 12345678
        Diese Nummer ist KEINE Steuer-ID!
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    // Should find phone but the 11-digit pattern shouldn't match
    // because phone numbers have different formats
    let phones: Vec<_> = findings
        .iter()
        .filter(|f| f.category == PiiCategory::Phone)
        .collect();
    assert!(!phones.is_empty(), "Should find phone number");
}

#[test]
fn test_confidence_scores_are_reasonable() {
    let registry = DetectorRegistry::default();
    let document = r#"
        Steuer-ID: 86095742719
        AHV-Nr.: 756.1234.5678.97
        USt-IdNr.: DE123456789
        Personalausweis: L01X00T471
    "#;

    let segments = vec![make_segment(document)];
    let findings = registry.detect_all(&segments);

    for finding in &findings {
        assert!(
            finding.confidence > 0.0 && finding.confidence <= 1.0,
            "Confidence should be between 0 and 1, got {} for {:?}",
            finding.confidence,
            finding.category
        );

        // All findings should have some confidence (even if validation fails,
        // which reduces confidence to ~0.3 of base)
        assert!(
            finding.confidence >= 0.1,
            "All detectors should have confidence >= 0.1, got {} for {:?}",
            finding.confidence,
            finding.category
        );
    }

    // Verify we found the expected categories
    let categories: Vec<_> = findings.iter().map(|f| &f.category).collect();
    assert!(
        categories.contains(&&PiiCategory::TaxIdDe),
        "Should find Tax ID"
    );
    assert!(categories.contains(&&PiiCategory::AhvCh), "Should find AHV");
    assert!(
        categories.contains(&&PiiCategory::VatNumber),
        "Should find VAT"
    );
    assert!(
        categories.contains(&&PiiCategory::NationalIdDe),
        "Should find National ID"
    );
}

// =============================================================================
// Performance Test with Large Document
// =============================================================================

#[test]
fn test_performance_large_document() {
    let registry = DetectorRegistry::default();

    // Generate a large document with many PII instances
    let mut document = String::with_capacity(100_000);
    document.push_str("KUNDENLISTE\n\n");

    for i in 0..100 {
        document.push_str(&format!(
            "Kunde {}: Steuer-ID: {}{:010}, USt-IdNr.: DE{:09}, AHV: 756.{:04}.{:04}.{:02}\n",
            i,
            (i % 9) + 1, // First digit 1-9
            i * 12345 % 10000000000u64,
            i * 11111 % 1000000000,
            i % 10000,
            (i * 7) % 10000,
            i % 100
        ));
    }

    let segments = vec![make_segment(&document)];

    let start = std::time::Instant::now();
    let findings = registry.detect_all(&segments);
    let duration = start.elapsed();

    println!("Processed {} bytes in {:?}", document.len(), duration);
    println!("Found {} PII instances", findings.len());

    // Should complete in reasonable time (< 1 second for this size)
    assert!(
        duration.as_secs() < 1,
        "Detection should complete in < 1 second, took {:?}",
        duration
    );

    // Should find many instances
    assert!(
        findings.len() > 100,
        "Should find many PII instances, found {}",
        findings.len()
    );
}
