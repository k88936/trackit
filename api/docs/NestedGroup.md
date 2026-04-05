# NestedGroup

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**name** | Option<**String**> |  | [optional][readonly]
**ring_id** | Option<**String**> |  | [optional][readonly]
**users_count** | Option<**i64**> |  | [optional][readonly]
**icon** | Option<**String**> |  | [optional][readonly]
**all_users_group** | Option<**bool**> |  | [optional][readonly]
**users** | Option<[**Vec<models::User>**](User.md)> |  | [optional][readonly]
**dollar_type** | Option<**String**> |  | [optional][readonly]
**parent_group** | Option<[**models::NestedGroup**](NestedGroup.md)> |  | [optional]
**sub_groups** | Option<[**models::NestedGroup**](NestedGroup.md)> |  | [optional]
**own_users** | Option<[**models::User**](User.md)> |  | [optional]
**require_two_factor_authentication** | Option<**bool**> |  | [optional]
**viewers** | Option<**serde_json::Value**> |  | [optional][readonly]
**updaters** | Option<**serde_json::Value**> |  | [optional][readonly]
**auto_join** | Option<**bool**> |  | [optional]
**auto_join_domain** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


