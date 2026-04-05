# Article

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> |  | [optional][readonly]
**attachments** | Option<[**Vec<models::ArticleAttachment>**](ArticleAttachment.md)> |  | [optional]
**content** | Option<**String**> |  | [optional]
**reporter** | Option<[**models::User**](User.md)> |  | [optional]
**summary** | Option<**String**> |  | [optional]
**visibility** | Option<[**models::Visibility**](Visibility.md)> |  | [optional]
**dollar_type** | Option<**String**> |  | [optional][readonly]
**child_articles** | Option<[**Vec<models::Article>**](Article.md)> |  | [optional]
**comments** | Option<[**Vec<models::ArticleComment>**](ArticleComment.md)> |  | [optional]
**created** | Option<**i64**> |  | [optional][readonly]
**external_article** | Option<[**models::ExternalArticle**](ExternalArticle.md)> |  | [optional]
**has_children** | Option<**bool**> |  | [optional][readonly]
**has_star** | Option<**bool**> |  | [optional]
**id_readable** | Option<**String**> |  | [optional][readonly]
**ordinal** | Option<**i64**> |  | [optional][readonly]
**parent_article** | Option<[**models::Article**](Article.md)> |  | [optional]
**pinned_comments** | Option<[**Vec<models::ArticleComment>**](ArticleComment.md)> |  | [optional][readonly]
**project** | Option<[**models::Project**](Project.md)> |  | [optional]
**tags** | Option<[**Vec<models::Tag>**](Tag.md)> |  | [optional]
**updated** | Option<**i64**> |  | [optional][readonly]
**updated_by** | Option<[**models::User**](User.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


