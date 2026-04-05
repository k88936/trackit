# Project

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**name** | Option<**String**> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]
**archived** | Option<**bool**> |  | [optional]
**created_by** | Option<[**models::User**](User.md)> |  | [optional]
**custom_fields** | Option<**serde_json::Value**> |  | [optional][readonly]
**description** | Option<**String**> |  | [optional]
**from_email** | Option<**String**> |  | [optional]
**icon_url** | Option<**String**> |  | [optional][readonly]
**issues** | Option<[**Vec<models::Issue>**](Issue.md)> |  | [optional]
**leader** | Option<[**models::User**](User.md)> |  | [optional]
**reply_to_email** | Option<**String**> |  | [optional]
**short_name** | Option<**String**> |  | [optional]
**starting_number** | Option<**i64**> |  | [optional]
**team** | Option<[**models::ProjectTeam**](ProjectTeam.md)> |  | [optional]
**template** | Option<**bool**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


