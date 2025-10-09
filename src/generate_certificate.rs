use openssl::{
    asn1::Asn1Time,
    pkey::PKey,
    rsa::Rsa,
    x509::{
        extension::{AuthorityKeyIdentifier, BasicConstraints, SubjectKeyIdentifier},
        X509NameBuilder, X509,
    },
};

pub fn generate_certificate() -> anyhow::Result<(String, String)> {
    let cert_result: Result<(Vec<u8>, Vec<u8>), openssl::error::ErrorStack> = {
        let rsa = Rsa::generate(4096)?;
        let pkey = PKey::from_rsa(rsa)?;
        let mut name = X509NameBuilder::new()?;
        name.append_entry_by_text("CN", "localhost")?;
        let name = name.build();
        let mut builder = X509::builder()?;
        builder.set_version(2)?;
        builder.set_subject_name(&name)?;
        builder.set_issuer_name(&name)?;
        builder.set_pubkey(&pkey)?;
        let now = Asn1Time::days_from_now(0)?;
        let later = Asn1Time::days_from_now(3650)?;
        builder.set_not_before(now.as_ref())?;
        builder.set_not_after(later.as_ref())?;
        builder.append_extension(BasicConstraints::new().critical().ca().build()?)?;
        builder.append_extension(
            SubjectKeyIdentifier::new().build(&builder.x509v3_context(None, None))?,
        )?;
        builder.append_extension(
            AuthorityKeyIdentifier::new()
                .keyid(true)
                .issuer(true)
                .build(&builder.x509v3_context(None, None))?,
        )?;
        builder.sign(&pkey, openssl::hash::MessageDigest::sha256())?;
        let c = builder.build();
        Ok((c.to_pem()?, pkey.private_key_to_pem_pkcs8()?))
    };
    let (pem_certificate, pem_private_key) = cert_result?;

    Ok((
        String::from_utf8_lossy(&pem_certificate).to_string(),
        String::from_utf8_lossy(&pem_private_key).to_string(),
    ))
}
