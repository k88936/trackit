# SavedQuery

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**name** | Option<**String**> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]
**owner** | Option<[**models::User**](User.md)> |  | [optional]
**visible_for** | Option<[**models::UserGroup**](UserGroup.md)> |  | [optional]
**updateable_by** | Option<[**models::UserGroup**](UserGroup.md)> |  | [optional]
**read_sharing_settings** | Option<[**models::WatchFolderSharingSettings**](WatchFolderSharingSettings.md)> |  | [optional]
**update_sharing_settings** | Option<[**models::WatchFolderSharingSettings**](WatchFolderSharingSettings.md)> |  | [optional]
**query** | Option<**String**> |  | [optional]
**issues** | Option<[**Vec<models::Issue>**](Issue.md)> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


