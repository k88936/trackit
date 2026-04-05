# OwnedProjectCustomField

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**field** | Option<[**models::CustomField**](CustomField.md)> |  | [optional]
**project** | Option<[**models::Project**](Project.md)> |  | [optional]
**can_be_empty** | Option<**bool**> |  | [optional]
**empty_field_text** | Option<**String**> |  | [optional]
**ordinal** | Option<**i32**> |  | [optional]
**is_public** | Option<**bool**> |  | [optional]
**has_running_job** | Option<**bool**> |  | [optional][readonly]
**condition** | Option<[**models::CustomFieldCondition**](CustomFieldCondition.md)> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]
**bundle** | Option<[**models::OwnedBundle**](OwnedBundle.md)> |  | [optional]
**default_values** | Option<[**Vec<models::OwnedBundleElement>**](OwnedBundleElement.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


