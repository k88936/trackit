# Agile

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**name** | Option<**String**> |  | [optional]
**owner** | Option<[**models::User**](User.md)> |  | [optional]
**visible_for** | Option<[**models::UserGroup**](UserGroup.md)> |  | [optional]
**visible_for_project_based** | Option<**bool**> |  | [optional]
**updateable_by** | Option<[**models::UserGroup**](UserGroup.md)> |  | [optional]
**updateable_by_project_based** | Option<**bool**> |  | [optional]
**read_sharing_settings** | Option<[**models::AgileSharingSettings**](AgileSharingSettings.md)> |  | [optional]
**update_sharing_settings** | Option<[**models::AgileSharingSettings**](AgileSharingSettings.md)> |  | [optional]
**orphans_at_the_top** | Option<**bool**> |  | [optional]
**hide_orphans_swimlane** | Option<**bool**> |  | [optional]
**estimation_field** | Option<[**models::CustomField**](CustomField.md)> |  | [optional]
**original_estimation_field** | Option<[**models::CustomField**](CustomField.md)> |  | [optional]
**projects** | Option<[**Vec<models::Project>**](Project.md)> |  | [optional]
**sprints** | Option<[**Vec<models::Sprint>**](Sprint.md)> |  | [optional]
**current_sprint** | Option<[**models::Sprint**](Sprint.md)> |  | [optional]
**column_settings** | Option<[**models::ColumnSettings**](ColumnSettings.md)> |  | [optional]
**swimlane_settings** | Option<[**models::SwimlaneSettings**](SwimlaneSettings.md)> |  | [optional]
**sprints_settings** | Option<[**models::SprintsSettings**](SprintsSettings.md)> |  | [optional]
**color_coding** | Option<[**models::ColorCoding**](ColorCoding.md)> |  | [optional]
**status** | Option<[**models::AgileStatus**](AgileStatus.md)> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


