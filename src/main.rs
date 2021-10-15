use ed25519_dalek::{Keypair, Signer, Verifier};
use rand::rngs::OsRng;

pub fn main() {
    let mut rng = OsRng {};
    let keypair: Keypair = Keypair::generate(&mut rng);
    let msg = b"hello world";

    println!("Public key: {:?}", &hex::encode(&keypair.public));
    println!("Private key: {:?}", &hex::encode(&keypair.secret));

    let signature = keypair.sign(msg);

    assert!(keypair.public.verify(msg, &signature).is_ok());
    assert!(keypair
        .public
        .verify(b"another message", &signature)
        .is_err());
}
