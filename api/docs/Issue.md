# Issue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**attachments** | Option<[**Vec<models::IssueAttachment>**](IssueAttachment.md)> |  | [optional]
**comments** | Option<[**Vec<models::IssueComment>**](IssueComment.md)> |  | [optional]
**comments_count** | Option<**i32**> |  | [optional][readonly]
**created** | Option<**i64**> |  | [optional][readonly]
**custom_fields** | Option<[**Vec<models::IssueCustomField>**](IssueCustomField.md)> |  | [optional][readonly]
**description** | Option<**String**> |  | [optional]
**draft_owner** | Option<[**models::User**](User.md)> |  | [optional]
**external_issue** | Option<[**models::ExternalIssue**](ExternalIssue.md)> |  | [optional]
**id_readable** | Option<**String**> |  | [optional][readonly]
**is_draft** | Option<**bool**> |  | [optional][readonly]
**links** | Option<[**Vec<models::IssueLink>**](IssueLink.md)> |  | [optional][readonly]
**number_in_project** | Option<**i64**> |  | [optional][readonly]
**parent** | Option<[**models::IssueLink**](IssueLink.md)> |  | [optional]
**pinned_comments** | Option<[**Vec<models::IssueComment>**](IssueComment.md)> |  | [optional][readonly]
**project** | Option<[**models::Project**](Project.md)> |  | [optional]
**reporter** | Option<[**models::User**](User.md)> |  | [optional]
**resolved** | Option<**i64**> |  | [optional][readonly]
**subtasks** | Option<[**models::IssueLink**](IssueLink.md)> |  | [optional]
**summary** | Option<**String**> |  | [optional]
**tags** | Option<[**Vec<models::Tag>**](Tag.md)> |  | [optional]
**updated** | Option<**i64**> |  | [optional][readonly]
**updater** | Option<[**models::User**](User.md)> |  | [optional]
**visibility** | Option<[**models::Visibility**](Visibility.md)> |  | [optional]
**voters** | Option<[**models::IssueVoters**](IssueVoters.md)> |  | [optional]
**votes** | Option<**i32**> |  | [optional][readonly]
**watchers** | Option<[**models::IssueWatchers**](IssueWatchers.md)> |  | [optional]
**wikified_description** | Option<**String**> |  | [optional][readonly]
**dollar_type** | Option<**String**> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


