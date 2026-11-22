use std::fs;
use std::path::Path;

/// Trusted root certificates of the operating system, for platforms where the
/// bundled OpenSSL cannot find them by itself.
pub struct SystemCertificates;

impl SystemCertificates {
    /// Concatenates the PEM files of the first directory that exists.
    pub fn read_from(stores: &[&Path]) -> Vec<u8> {
        stores
            .iter()
            .find_map(|store| fs::read_dir(store).ok())
            .map(concatenate)
            .unwrap_or_default()
    }

    /// Makes Git trust the Android system certificates and returns how many were added.
    #[cfg(target_os = "android")]
    pub fn trust_android_store() -> usize {
        let pem = Self::read_from(&[
            Path::new("/apex/com.android.conscrypt/cacerts"),
            Path::new("/system/etc/security/cacerts"),
        ]);
        trust(&pem)
    }
}

fn concatenate(entries: fs::ReadDir) -> Vec<u8> {
    let mut paths: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    paths
        .iter()
        .filter_map(|path| fs::read(path).ok())
        .flat_map(|mut certificate| {
            certificate.push(b'\n');
            certificate
        })
        .collect()
}

#[cfg(target_os = "android")]
fn trust(pem: &[u8]) -> usize {
    libgit2_sys::init();
    let Ok(length) = i32::try_from(pem.len()) else {
        return 0;
    };
    let mut added = 0;
    unsafe {
        let bio = openssl_sys::BIO_new_mem_buf(pem.as_ptr().cast(), length);
        loop {
            let certificate = openssl_sys::PEM_read_bio_X509(
                bio,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
            );
            if certificate.is_null() {
                break;
            }
            let option = libgit2_sys::GIT_OPT_ADD_SSL_X509_CERT as std::ffi::c_int;
            if libgit2_sys::git_libgit2_opts(option, certificate) == 0 {
                added += 1;
            }
            openssl_sys::X509_free(certificate);
        }
        openssl_sys::ERR_clear_error();
        openssl_sys::BIO_free_all(bio);
    }
    added
}
