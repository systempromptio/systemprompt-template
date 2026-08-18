 # dw.order.PaymentInstrument

 ## Overview
 Base class for payment instruments stored in customer profiles or related to orders; supports credit card and bank transfer types.

 ## Description
 Base class for payment instrument either stored in the customers profile or related to an order. A payment instrument can be credit card or bank transfer. The object defines standard methods for credit card payment, and can be extended by attributes appropriate for other payment methods.

 ## All Known Subclasses
 CustomerPaymentInstrument, OrderPaymentInstrument

 ```ts
 declare class PaymentInstrument extends dw.customer.EncryptedObject {
 	/** The outdated encryption algorithm "RSA/ECB/PKCS1Padding". */
 	static ENCRYPTION_ALGORITHM_RSA: 'RSA'

 	/** The encryption algorithm "RSA/ECB/OAEPWithSHA-256AndMGF1Padding". */
 	static ENCRYPTION_ALGORITHM_RSA_ECB_OAEPWITHSHA_256ANDMGF1PADDING: 'RSA/ECB/OAEPWithSHA-256AndMGF1Padding'

 	/** Represents a bank transfer payment method. */
 	static METHOD_BANK_TRANSFER: 'BANK_TRANSFER'

 	/** Represents a 'bill me later' payment method. */
 	static METHOD_BML: 'BML'

 	/** Represents a credit card payment method. */
 	static METHOD_CREDIT_CARD: 'CREDIT_CARD'

 	/** Represents an Android Pay payment. */
 	static METHOD_DW_ANDROID_PAY: 'DW_ANDROID_PAY'

 	/** Represents an Apple Pay payment. */
 	static METHOD_DW_APPLE_PAY: 'DW_APPLE_PAY'

 	/** Represents a gift certificate payment. */
 	static METHOD_GIFT_CERTIFICATE: 'GIFT_CERTIFICATE'

 	/** The driver's license number associated with the bank account (may be masked). */
 	bankAccountDriversLicense: string

 	/** The last 4 characters of the decrypted driver's license number (read-only). */
 	/** @readonly */
 	bankAccountDriversLicenseLastDigits: string

 	/** The driver's license state code associated with a bank account payment instrument. */
 	bankAccountDriversLicenseStateCode: string

 	/** The full name of the holder of a bank account payment instrument. */
 	bankAccountHolder: string

 	/** The bank account number (may be masked). */
 	bankAccountNumber: string

 	/** The last 4 characters of the decrypted bank account number (read-only). */
 	/** @readonly */
 	bankAccountNumberLastDigits: string

 	/** The bank routing number of a bank account payment instrument. */
 	bankRoutingNumber: string

 	/** The month (1-12) of credit card expiration. */
 	creditCardExpirationMonth: number

 	/** The year of credit card expiration (e.g., 2004). */
 	creditCardExpirationYear: number

 	/** Returns true if this payment instrument represents an expired credit card (read-only). */
 	/** @readonly */
 	creditCardExpired: boolean

 	/** The name of the credit card owner. */
 	creditCardHolder: string

 	/** Credit card issue number (where applicable). */
 	creditCardIssueNumber: string

 	/** The decrypted credit card number (may be masked). */
 	creditCardNumber: string

 	/** The last 4 digits of the decrypted credit card number (read-only). */
 	/** @readonly */
 	creditCardNumberLastDigits: string

 	/** The credit card type (e.g., VISA, MC). */
 	creditCardType: string

	/** Token for the credit card when stored via tokenization (read-only). */
	/** @readonly */
	creditCardToken: string

	/** The month (1-12) the credit card is valid from. */
	creditCardValidFromMonth: number

	/** The year the credit card is valid from. */
	creditCardValidFromYear: number

	/** Gift certificate code when this instrument represents a gift certificate. */
	giftCertificateCode: string

	/** Masked representations of sensitive fields for safe display. */
	maskedBankAccountDriversLicense: string

	/** Masked bank account number (read-only). */
	/** @readonly */
	maskedBankAccountNumber: string

	/** Masked credit card number (read-only). */
	/** @readonly */
	maskedCreditCardNumber: string

	/** Masked gift certificate code (read-only). */
	/** @readonly */
	maskedGiftCertificateCode: string

 	/** The customer profile ID that owns this instrument (read-only). */
 	/** @readonly */
 	customerProfileId: string

 	/** The payment method ID associated with this instrument. */
 	paymentMethodId: string

 	/** The token representing the payment instrument (read-only). */
 	/** @readonly */
 	paymentToken: string

 	/** Returns whether the payment instrument is masked (read-only). */
 	/** @readonly */
 	masked: boolean

 	/** Returns the IP address related to this payment (read-only). */
 	/** @readonly */
 	ipAddress: string

 	/** Returns the parent payment instrument ID if any (read-only). */
 	/** @readonly */
 	parentPaymentInstrumentId: string

 	/** Returns whether this instrument is the default for the customer (read-only). */
 	/** @readonly */
 	default: boolean

 	/** Returns a JSON-like representation of sensitive fields masked for safe display. */
 	getMaskedAccountNumber(): string

 	/** Returns true if the instrument supports the specified operation. */
 	supportsOperation(operationId: string): boolean

 }
 ```
