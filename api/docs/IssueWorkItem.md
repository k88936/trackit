# IssueWorkItem

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**dollar_type** | Option<**String**> |  | [optional][readonly]
**author** | Option<[**models::User**](User.md)> |  | [optional]
**creator** | Option<[**models::User**](User.md)> |  | [optional]
**text** | Option<**String**> |  | [optional]
**text_preview** | Option<**String**> |  | [optional][readonly]
**r#type** | Option<[**models::WorkItemType**](WorkItemType.md)> |  | [optional]
**created** | Option<**i64**> |  | [optional]
**updated** | Option<**i64**> |  | [optional]
**duration** | Option<[**models::DurationValue**](DurationValue.md)> |  | [optional]
**date** | Option<**i64**> |  | [optional]
**issue** | Option<[**models::Issue**](Issue.md)> |  | [optional]
**attributes** | Option<[**Vec<models::WorkItemAttribute>**](WorkItemAttribute.md)> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


