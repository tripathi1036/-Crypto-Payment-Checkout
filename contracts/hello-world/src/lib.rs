#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, Address, symbol_short, String};

// Payment record data structure
#[contracttype]
#[derive(Clone)]
pub struct PaymentRecord {
    pub payment_id: u64,
    pub payer: Address,
    pub merchant: Address,
    pub amount: u64,
    pub description: String,
    pub timestamp: u64,
    pub status: PaymentStatus,
}

// Payment status enum
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Refunded,
    Canceled,
}

// Contract storage keys
const PAYMENT_COUNT: Symbol = symbol_short!("PMT_COUNT");
const MERCHANT_DATA: Symbol = symbol_short!("MRCH_DATA");

// Mapping payment ID to payment record
#[contracttype]
pub enum PaymentMap {
    Payment(u64)
}

// Merchant data structure
#[contracttype]
#[derive(Clone)]
pub struct MerchantData {
    pub merchant_address: Address,
    pub total_payments: u64,
    pub total_volume: u64,
}

#[contract]
pub struct CryptoPaymentContract;

#[contractimpl]
impl CryptoPaymentContract {
    // Create a new payment
    pub fn create_payment(env: Env, payer: Address, merchant: Address, amount: u64, description: String) -> u64 {
        // Get current payment count
        let mut payment_count = env.storage().instance().get(&PAYMENT_COUNT).unwrap_or(0);
        payment_count += 1;
        
        // Create payment record
        let payment = PaymentRecord {
            payment_id: payment_count,
            payer: payer,
            merchant: merchant.clone(),
            amount: amount,
            description: description,
            timestamp: env.ledger().timestamp(),
            status: PaymentStatus::Pending,
        };
        
        // Store the payment record
        env.storage().instance().set(&PaymentMap::Payment(payment_count), &payment);
        env.storage().instance().set(&PAYMENT_COUNT, &payment_count);
        
        // Update merchant data
        let mut merchant_data = Self::get_merchant_data(env.clone(), merchant.clone());
        merchant_data.merchant_address = merchant;
        merchant_data.total_payments += 1;
        env.storage().instance().set(&MERCHANT_DATA, &merchant_data);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Payment {} created with amount {}", payment_count, amount);
        
        payment_count
    }
    
    // Complete a payment
    pub fn complete_payment(env: Env, payment_id: u64) -> PaymentRecord {
        // Get payment record
        let mut payment: PaymentRecord = env.storage().instance().get(&PaymentMap::Payment(payment_id))
            .unwrap_or_else(|| panic!("Payment not found"));
        
        // Check if payment is pending
        if payment.status != PaymentStatus::Pending {
            log!(&env, "Payment is not in pending status");
            panic!("Payment is not in pending status");
        }
        
        // Update payment status
        payment.status = PaymentStatus::Completed;
        env.storage().instance().set(&PaymentMap::Payment(payment_id), &payment);
        
        // Update merchant data
        let mut merchant_data = Self::get_merchant_data(env.clone(), payment.merchant.clone());
        merchant_data.total_volume += payment.amount;
        env.storage().instance().set(&MERCHANT_DATA, &merchant_data);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Payment {} completed", payment_id);
        
        payment
    }
    
    // Get payment details
    pub fn get_payment(env: Env, payment_id: u64) -> PaymentRecord {
        env.storage().instance().get(&PaymentMap::Payment(payment_id))
            .unwrap_or_else(|| panic!("Payment not found"))
    }
    
    // Get merchant data
    pub fn get_merchant_data(env: Env, merchant: Address) -> MerchantData {
        env.storage().instance().get(&MERCHANT_DATA).unwrap_or(MerchantData {
            merchant_address: merchant,
            total_payments: 0,
            total_volume: 0,
        })
    }
}