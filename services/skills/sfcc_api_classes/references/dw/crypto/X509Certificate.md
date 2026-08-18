 # dw.crypto.X509Certificate

 ## Overview
 Represents an X.509 public key certificate and exposes standard certificate fields and helpers.

 ## Description
 Provides access to certificate metadata: issuer/subject distinguished names, validity period, serial number, signature algorithm, and version. Instances are returned from certificate-related APIs and cannot be constructed directly.

 ```ts
 declare class X509Certificate extends CertificateRef {
     /** The issuer distinguished name (X.500) */
     issuerDN: string

     /** The certificate end date */
     notAfter: Date

     /** The certificate start date */
     notBefore: Date

     /** Serial number as string */
     serialNumber: string

     /** Signature algorithm name (e.g., "SHA256withRSA") */
     sigAlgName: string

     /** Subject distinguished name (X.500) */
     subjectDN: string

     /** Certificate version number */
     version: number

     getIssuerDN(): string
     getNotAfter(): Date
     getNotBefore(): Date
     getSerialNumber(): string
     getSigAlgName(): string
     getSubjectDN(): string
     getVersion(): number
 }
 ```
