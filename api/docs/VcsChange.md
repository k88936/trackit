# VcsChange

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**date** | Option<**i64**> |  | [optional][readonly]
**fetched** | Option<**i64**> |  | [optional][readonly]
**files** | Option<**i32**> |  | [optional][readonly]
**author** | Option<[**models::User**](User.md)> |  | [optional]
**processors** | Option<[**Vec<models::ChangesProcessor>**](ChangesProcessor.md)> |  | [optional][readonly]
**text** | Option<**String**> |  | [optional][readonly]
**urls** | Option<**Vec<String>**> |  | [optional][readonly]
**version** | Option<**String**> |  | [optional]
**issue** | Option<[**models::Issue**](Issue.md)> |  | [optional]
**state** | Option<**i32**> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


