# EmailSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**is_enabled** | Option<**bool**> |  | [optional]
**host** | Option<**String**> |  | [optional]
**port** | Option<**i32**> |  | [optional]
**mail_protocol** | Option<**MailProtocol**> |  (enum: SMTP, SMTPS, SMTP_TLS, MS_GRAPH_API) | [optional]
**anonymous** | Option<**bool**> |  | [optional]
**login** | Option<**String**> |  | [optional]
**ssl_key** | Option<[**models::StorageEntry**](StorageEntry.md)> |  | [optional]
**from** | Option<**String**> |  | [optional]
**reply_to** | Option<**String**> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


