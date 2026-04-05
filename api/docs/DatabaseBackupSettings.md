# DatabaseBackupSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**location** | Option<**String**> |  | [optional]
**files_to_keep** | Option<**i32**> |  | [optional]
**cron_expression** | Option<**String**> |  | [optional]
**archive_format** | Option<**ArchiveFormat**> |  (enum: TAR_GZ, ZIP) | [optional]
**is_on** | Option<**bool**> |  | [optional]
**available_disk_space** | Option<**i64**> |  | [optional][readonly]
**notified_users** | Option<[**Vec<models::User>**](User.md)> |  | [optional]
**backup_status** | Option<[**models::BackupStatus**](BackupStatus.md)> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


