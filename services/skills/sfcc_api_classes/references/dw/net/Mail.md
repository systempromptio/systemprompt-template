# dw.net.Mail

## Overview
Helper for composing and queuing emails (addresses, subject, content, attachments). Provides validation and queuing to the internal mail system.

## Description
Construct messages with recipients, subject and content (string or MimeEncodedText), add attachments and headers, then call send() to enqueue for delivery. Some operations may be restricted to Job context.

```ts
declare class Mail  {
    constructor()

    /** Adds a file attachment (job context only). */
    addAttachment(file: File): Mail

    /** Adds a BCC address. */
    addBcc(bcc: string): Mail

    /** Adds a CC address. */
    addCc(cc: string): Mail

    /** Adds a Reply-To address. */
    addReplyTo(replyTo: string): Mail

    /** Adds a To address. */
    addTo(to: string): Mail

    /** Gets the BCC list. */
    getBcc(): List

    /** Gets the CC list. */
    getCc(): List

    /** Gets the From address. */
    getFrom(): string

    /** Gets the Reply-To list. */
    getReplyTo(): List

    /** Gets the Subject. */
    getSubject(): string

    /** Gets the To list. */
    getTo(): List

    /** Prepares and queues the email for delivery; returns Status.OK or Status.ERROR. */
    send(): Status

    /** Sets the BCC list (replaces existing). */
    setBcc(bcc: List): Mail

    /** Sets the CC list (replaces existing). */
    setCc(cc: List): Mail

    /** Sets the email content (mandatory). */
    setContent(content: string): Mail

    /** Sets the email content with MIME type and encoding (mandatory). */
    setContent(content: string, mimeType: string, encoding: string): Mail

    /** Sets content from a MimeEncodedText object. */
    setContent(mimeEncodedText: MimeEncodedText): Mail

    /** Sets the From address (mandatory). */
    setFrom(from: string): Mail

    /** Sets the List-Unsubscribe header. */
    setListUnsubscribe(listUnsubscribe: string): Mail

    /** Sets the List-Unsubscribe-Post header. */
    setListUnsubscribePost(listUnsubscribePost: string): Mail

    /** Sets the email subject (mandatory). */
    setSubject(subject: string): Mail

    /** Sets the To list (replaces existing). */
    setTo(to: List): Mail

    /** Validates an RFC822 address; returns true when valid. */
    static validateAddress(address: string): boolean
}
```
