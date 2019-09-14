use parity_codec::Encode;
use support::{StorageValue, dispatch::Result, decl_module, decl_storage};
use support::traits::{Currency, WithdrawReason, ExistenceRequirement};
use runtime_primitives::traits::{Zero, Hash, Saturating};
use system::ensure_signed;

pub trait Trait: balances::Trait {}

decl_storage! {
  trait Store for Module<T: Trait> as Demo {
    Payable get(payable): Option<T::Balance>;
    Pot get(pot): T::Balance;
    Nonce get(nonce): u64;
  }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn init_payment(origin, value: T::Balance) -> Result {
        let _ = ensure_signed(origin)?;

        if Self::payable().is_none() {
            <Payable<T>>::put(value);
            <Pot<T>>::put(value);
        }

        Ok(())
    }

    fn upload(origin) -> Result {
      let sender = ensure_signed(origin)?;

      let payable = Self::payable().ok_or("Must have payable amount set")?;

      let mut nonce = Self::nonce();
      let mut pot = Self::pot();

      // Try to withdraw the payable from the account, making sure that it will not kill the account
      let _ = <balances::Module<T> as Currency<_>>::withdraw(&sender, payable, WithdrawReason::Reserve, ExistenceRequirement::KeepAlive)?;

      // Generate a random hash between 0-255 using a csRNG algorithm
      if (<system::Module<T>>::random_seed(), &sender, nonce)
        .using_encoded(<T as system::Trait>::Hashing::hash)
        .using_encoded(|e| e[0] < 128)
        {
          // If the user won the coin flip, deposit the pot winnings; cannot fail
          let _ = <balances::Module<T> as Currency<_>>::deposit_into_existing(&sender, pot)
            .expect("`sender` must exist since a transaction is being made and withdraw will keep alive; qed.");

          // Reduce the pot to zero
          pot = Zero::zero();
      }

      // No matter the outcome, increase the pot by the payable amount
      pot = pot.saturating_add(payable);

      nonce = nonce.wrapping_add(1);

      // Store the updated values for our module
      <Pot<T>>::put(pot);
      <Nonce<T>>::put(nonce);

      Ok(())
    }
    }
}