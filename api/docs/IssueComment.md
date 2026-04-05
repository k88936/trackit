# IssueComment

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**attachments** | Option<[**Vec<models::IssueAttachment>**](IssueAttachment.md)> |  | [optional]
**author** | Option<[**models::User**](User.md)> |  | [optional]
**created** | Option<**i64**> |  | [optional][readonly]
**deleted** | Option<**bool**> |  | [optional]
**issue** | Option<[**models::Issue**](Issue.md)> |  | [optional]
**pinned** | Option<**bool**> |  | [optional]
**reactions** | Option<[**Vec<models::Reaction>**](Reaction.md)> |  | [optional]
**text** | Option<**String**> |  | [optional]
**text_preview** | Option<**String**> |  | [optional][readonly]
**updated** | Option<**i64**> |  | [optional][readonly]
**visibility** | Option<[**models::Visibility**](Visibility.md)> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


