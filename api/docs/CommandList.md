# CommandList

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**comment** | Option<**String**> |  | [optional]
**visibility** | Option<[**models::CommandVisibility**](CommandVisibility.md)> |  | [optional]
**query** | Option<**String**> |  | [optional]
**caret** | Option<**i32**> |  | [optional]
**silent** | Option<**bool**> |  | [optional]
**run_as** | Option<**String**> |  | [optional]
**commands** | Option<[**Vec<models::ParsedCommand>**](ParsedCommand.md)> |  | [optional][readonly]
**issues** | Option<[**Vec<models::Issue>**](Issue.md)> |  | [optional]
**suggestions** | Option<[**Vec<models::Suggestion>**](Suggestion.md)> |  | [optional][readonly]
**dollar_type** | Option<**String**> |  | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


