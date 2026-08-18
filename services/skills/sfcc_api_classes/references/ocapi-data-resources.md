# OCAPI Data API — Resource Reference

Base URL: `https://{host}/s/-/dw/data/v24_5`

## Products & Search

| Path | Methods |
|---|---|
| `/products/{id}` | GET, DELETE, PUT, PATCH |
| `/products/{master_product_id}/variation_groups` | GET, PUT, PATCH, DELETE |
| `/products/{product_id}/product_options/{option_id}/values/{id}` | GET, PUT, PATCH, DELETE |
| `/product_search` | POST |
| `/variant_search` | POST |

## Catalogs & Categories

| Path | Methods |
|---|---|
| `/catalogs` | GET, POST |
| `/catalogs/{catalog_id}` | GET, PATCH, PUT, DELETE |
| `/catalogs/{catalog_id}/category_search` | POST |
| `/catalogs/{catalog_id}/shared_product_options` | GET, PUT, PATCH, DELETE |
| `/catalog_search` | POST |
| `/catalogs/{catalog_id}/categories` | GET, PUT |
| `/catalogs/{catalog_id}/categories/{category_id}` | GET, PUT, PATCH, DELETE |
| `/catalogs/{catalog_id}/categories/{category_id}/products/{product_id}` | PUT, DELETE |
| `/category_search` | POST |
| `/catalogs/{catalog_id}/categories/{category_id}/category_links` | GET |
| `/category_product_assignment_search` | POST |

## Customers

| Path | Methods |
|---|---|
| `/customer_lists/{list_id}` | GET, PATCH, DELETE, PUT |
| `/customer_lists/{list_id}/customer_search` | POST |
| `/customer_lists/{list_id}/customers` | GET, PUT |
| `/customer_lists/{list_id}/customers/{customer_no}` | GET, PUT, PATCH, DELETE |
| `/customer_lists/{list_id}/customers/{customer_no}/addresses` | GET, PUT, PATCH, DELETE |
| `/sites/{site_id}/customer_groups` | GET, PUT |
| `/sites/{site_id}/customer_groups/{id}` | GET, PATCH, DELETE, PUT |
| `/sites/{site_id}/customer_groups/{id}/member_search` | POST |
| `/sites/{site_id}/customer_groups/{id}/members/{customer_no}` | PUT, DELETE |
| `/sites/{site_id}/customer_group_search` | POST |

Note: CustomerLists requires knowing the `list_id`. There is no "list all customer lists" endpoint. Use site archive export if you need to discover list IDs.

## Inventory

| Path | Methods |
|---|---|
| `/inventory_lists` | GET, PUT |
| `/inventory_lists/{id}` | GET, PATCH, DELETE, PUT |
| `/inventory_list_search` | POST |
| `/inventory_lists/{inventory_list_id}/product_inventory_records` | GET, PUT |
| `/inventory_lists/{inventory_list_id}/product_inventory_records/{product_id}` | GET, PATCH, DELETE, PUT |

## Sites & Preferences

| Path | Methods |
|---|---|
| `/sites` | GET |
| `/sites/{site_id}` | GET, PATCH, PUT, DELETE |
| `/sites/{site_id}/cartridges` | GET |
| `/site_search` | POST |
| `/sites/{site_id}/site_preferences/preference_groups/{group_id}/{instance_type}` | GET, PUT, PATCH, DELETE |
| `/global_preferences/preference_groups/{group_id}/{instance_type}` | GET, PATCH |

## Jobs & Code Versions

| Path | Methods |
|---|---|
| `/jobs/{job_id}/executions` | POST, GET |
| `/jobs/{job_id}/executions/{id}` | GET, DELETE |
| `/job_execution_search` | POST |
| `/code_versions` | GET, PUT |
| `/code_versions/{code_version_id}` | GET, PATCH, DELETE |

## Promotions & Coupons

| Path | Methods |
|---|---|
| `/sites/{site_id}/promotions/{id}` | GET, PATCH, DELETE, PUT |
| `/sites/{site_id}/promotion_search` | POST |
| `/sites/{site_id}/coupons` | GET, PUT |
| `/sites/{site_id}/coupons/{coupon_id}` | GET, PATCH, DELETE, PUT |
| `/sites/{site_id}/coupons/{coupon_id}/codes` | GET, PUT |
| `/sites/{site_id}/coupons/{coupon_id}/multiple_codes` | POST |
| `/sites/{site_id}/coupon_search` | POST |
| `/sites/{site_id}/coupon_redemption_search` | POST |

## Campaigns

| Path | Methods |
|---|---|
| `/sites/{site_id}/campaigns/{campaign_id}` | GET, PUT, PATCH, DELETE |
| `/sites/{site_id}/campaigns/{campaign_id}/coupons/{coupon_id}` | PUT, DELETE |
| `/sites/{site_id}/campaigns/{campaign_id}/promotions/{promotion_id}` | PUT, PATCH, DELETE |
| `/sites/{site_id}/campaigns/{campaign_id}/slot_configurations/{slot_id}/{slot_config_id}` | PUT, PATCH, DELETE |
| `/sites/{site_id}/campaign_search` | POST |

## Custom Objects

| Path | Methods |
|---|---|
| `/custom_objects/{object_type}/{key}` | GET, PUT, DELETE, PATCH |
| `/sites/{site_id}/custom_objects/{object_type}/{key}` | GET, PUT, DELETE, PATCH |
| `/custom_objects_search/{object_type}` | POST |
| `/custom_object_definitions` | GET |
| `/custom_object_definitions/{object_type}` | GET, PUT, PATCH, DELETE |

## Stores

| Path | Methods |
|---|---|
| `/sites/{site_id}/stores` | GET, PUT |
| `/sites/{site_id}/stores/{id}` | GET, PATCH, DELETE, PUT |
| `/sites/{site_id}/store_search` | POST |

## Libraries (Content)

| Path | Methods |
|---|---|
| `/libraries/{library_id}/content/{content_id}` | GET, PUT, DELETE, PATCH |
| `/libraries/{library_id}/folders/{folder_id}` | GET, PUT, DELETE, PATCH |
| `/libraries/{library_id}/folders/{folder_id}/content` | GET, PUT |
| `/libraries/{library_id}/folders/{folder_id}/sub_folders` | GET, PUT |

## Slots & Slot Configurations

| Path | Methods |
|---|---|
| `/sites/{site_id}/slots` | GET |
| `/sites/{site_id}/slots/{slot_id}/{context_type}` | GET |
| `/sites/{site_id}/slots/{slot_id}/slot_configurations/{configuration_id}` | PUT, PATCH, GET, DELETE |
| `/sites/{site_id}/slot_search` | POST |
| `/sites/{site_id}/slot_configuration_search` | POST |

## Gift Certificates

| Path | Methods |
|---|---|
| `/sites/{site_id}/gift_certificates` | GET, POST |
| `/sites/{site_id}/gift_certificates/{merchant_id}` | GET, PATCH, DELETE |
| `/sites/{site_id}/gift_certificate_search` | POST |

## Source Code Groups

| Path | Methods |
|---|---|
| `/sites/{site_id}/source_code_groups` | GET |
| `/sites/{site_id}/source_code_groups/{id}` | GET, PUT, PATCH, DELETE |
| `/sites/{site_id}/source_code_group_search` | POST |

## Users & Roles

| Path | Methods |
|---|---|
| `/users` | GET |
| `/users/{login}` | GET, PUT, PATCH, DELETE |
| `/user_search` | POST |
| `/roles` | GET |
| `/roles/{id}` | GET, PUT, PATCH, DELETE |
| `/role_search` | POST |

## System Object Definitions

| Path | Methods |
|---|---|
| `/system_object_definitions` | GET |
| `/system_object_definitions/{object_type}` | GET, PATCH, DELETE, PUT |
| `/system_object_definitions/{object_type}/attribute_definitions` | GET, POST |
| `/system_object_definitions/{object_type}/attribute_definitions/{id}` | GET, PATCH, DELETE, PUT |
| `/system_object_definition_search` | POST |

## OCAPI Config

| Path | Methods |
|---|---|
| `/ocapi_configs/{clientId}` | GET, PUT, POST, DELETE |

## Other

| Path | Methods |
|---|---|
| `/ab_tests/{site_id}/{ab_test_id}` | GET, PUT, PATCH, DELETE |
| `/ab_test_search` | POST |
| `/locale_info/locales` | GET |
| `/log_requests` | POST |
| `/sorting_rule_search` | POST |
| `/promotion_campaign_assignment_search` | POST |
| `/slot_configuration_campaign_assignment_search` | POST |

---

## NOT in OCAPI Data API

These resources have NO endpoint in OCAPI. Do NOT attempt to call them:

| Resource | Alternative |
|---|---|
| **Price Books** | Site archive export: `data_units.price_books` |
| **Assignments** (price book to site) | SCAPI `product/assignments/v1` |
| **Experiences** (Page Designer) | SCAPI `experience/experiences/v1` |
| **CDN Zones** | SCAPI `cdn/zones/v1` |
| **CORS** | SCAPI `cors/v1` |
| **Consents** | SCAPI `consents/v1` |
| **SEO** (URL rules, redirects) | SCAPI `seo/v1` |
| **Timeouts** | SCAPI `timeouts/v1` |
| **Store Redirect Mappings** | SCAPI `store-redirect-mappings/v1` |

## Site Archive Export Fallback

When SCAPI scope is missing, use `sfcc-site-archive-export` job:

```json
{
  "export_file": "<name>.zip",
  "overwrite_export_file": true,
  "data_units": {
    "price_books": { "all": true },
    "catalogs": { "<catalog-id>": true },
    "inventory_lists": { "all": true }
  }
}
```

`price_books`, `catalogs`, `inventory_lists`, `customer_lists`, `libraries` are **top-level** in `data_units` (NOT inside `global_data` or `sites`).

Download result via WebDAV: `GET /on/demandware.servlet/webdav/Sites/Impex/src/instance/<name>.zip`
