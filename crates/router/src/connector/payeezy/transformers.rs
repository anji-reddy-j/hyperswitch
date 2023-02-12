use crate::{
    core::errors,
    types::{self, api, storage::enums}, utils::OptionExt, connector::utils::{CardData, PaymentsRequestData},
};
use common_utils::ext_traits::ValueExt;
use error_stack::{IntoReport, ResultExt};
// use actix_http::ws::Item;
// use frunk::labelled::chars::O;
use serde::{Deserialize, Serialize};

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, Eq, PartialEq)]
pub struct PayeezyPaymentsRequest {
    pub merchant_ref: String,
    pub transaction_type: String,
    pub method: String,
    pub amount: String,
    pub currency_code: String,
    pub credit_card: PayeezyCreditCard,
}

#[derive(Default, Debug, Eq, PartialEq, Serialize)]
pub struct PayeezyCreditCard {
    #[serde(rename = "type")]
    pub crad_type: String,
    pub cardholder_name: String,
    pub card_number: String,
    pub exp_date: String,
    pub cvv: String,
}

impl TryFrom<&types::PaymentsAuthorizeRouterData> for PayeezyPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_item: &types::PaymentsAuthorizeRouterData) -> Result<Self, Self::Error> {
        let merchant_ref = "Dummy Refrence".to_string();
        let transaction_type = "authorize".to_string();
        let method = "credit_card".to_string();
        let amount = _item.request.amount.to_string();
        let currency_code = _item.request.currency.to_string();
        let c = _item.get_card()?;
        let credit_card = PayeezyCreditCard {
                crad_type : "visa".to_string(),
                cardholder_name: c.get_card_holder_name(),
                card_number: c.get_card_number(),
                exp_date: format!("{}{}",c.get_card_expiry_month(),c.get_card_expiry_year_2_digit()),
                cvv: c.get_card_number(),
            };

    Ok(Self {
        merchant_ref,
        transaction_type,
        method,
        amount,
        currency_code,
        credit_card,
    })
    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct PayeezyAuthType {
    pub(super) api_key: String,
    pub(super) token: String
}

impl TryFrom<&types::ConnectorAuthType> for PayeezyAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        if let types::ConnectorAuthType::BodyKey { api_key, key1 } = _auth_type {
            Ok(Self {
                api_key: api_key.to_string(),
                token: key1.to_string(),
            })
        } else {
            Err(errors::ConnectorError::FailedToObtainAuthType)?
        }
    }
}
// PaymentsResponse
//TODO: Append the remaining status flags
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PayeezyPaymentStatus {
    #[serde(rename = "success")]
    Succeeded,
    #[serde(rename = "failure")]
    Failed,
    #[default]
    Processing,
}

impl From<PayeezyPaymentStatus> for enums::AttemptStatus {
    fn from(item: PayeezyPaymentStatus) -> Self {
        match item {
            PayeezyPaymentStatus::Succeeded => Self::Authorized,
            PayeezyPaymentStatus::Failed => Self::Failure,
            PayeezyPaymentStatus::Processing => Self::Authorizing,
        }
    }
}

//TODO: Fill the struct with respective fields
// #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct PayeezyPaymentsResponse {
//     status: PayeezyPaymentStatus,
//     id: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeezyPaymentsResponse {
    #[serde(rename = "correlation_id")]
    pub correlation_id: String,
    #[serde(rename = "transaction_status")]
    pub transaction_status: PayeezyPaymentStatus,
    #[serde(rename = "validation_status")]
    pub validation_status: String,
    #[serde(rename = "transaction_type")]
    pub transaction_type: String,
    #[serde(rename = "transaction_id")]
    pub transaction_id: String,
    #[serde(rename = "transaction_tag")]
    pub transaction_tag: String,
    pub method: String,
    pub amount: String,
    pub currency: String,
    pub cvv2: String,
    pub token: Token,
    pub card: Card,
    #[serde(rename = "bank_resp_code")]
    pub bank_resp_code: String,
    #[serde(rename = "bank_message")]
    pub bank_message: String,
    #[serde(rename = "gateway_resp_code")]
    pub gateway_resp_code: String,
    #[serde(rename = "gateway_message")]
    pub gateway_message: String,
    #[serde(rename = "retrieval_ref_no")]
    pub retrieval_ref_no: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    #[serde(rename = "token_type")] 
    pub token_type: String,
    #[serde(rename = "token_data")]
    pub token_data: TokenData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenData {
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "cardholder_name")]
    pub cardholder_name: String,
    #[serde(rename = "card_number")]
    pub card_number: String,
    #[serde(rename = "exp_date")]
    pub exp_date: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayeezyMeta {
    pub tag: String,
    pub amount: String,
    pub currency: String,
}

impl<F, T>
    TryFrom<types::ResponseRouterData<F, PayeezyPaymentsResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        item: types::ResponseRouterData<F, PayeezyPaymentsResponse, T, types::PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            status: enums::AttemptStatus::from(item.response.transaction_status),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.transaction_id),
                redirection_data: None,
                redirect: false,
                mandate_reference: None,
                connector_metadata: Some(serde_json::to_value(PayeezyMeta{ 
                    tag: item.response.transaction_tag.to_string(),
                    amount: item.response.amount,
                    currency: item.response.currency
                }).into_report().change_context(errors::ParsingError)?, ),
            }),
            ..item.data
        })
    }
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeezyVoidRequest {
    #[serde(rename = "merchant_ref")]
    pub merchant_ref: String,
    #[serde(rename = "transaction_tag")]
    pub transaction_tag: String,
    #[serde(rename = "transaction_type")]
    pub transaction_type: String,
    pub method: String,
    pub amount: String,
    #[serde(rename = "currency_code")]
    pub currency_code: String,
}

impl TryFrom<&types::PaymentsCancelRouterData> for PayeezyVoidRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_item: &types::PaymentsCancelRouterData) -> Result<Self, Self::Error> {
        let meta: PayeezyMeta = _item.request.connector_metadata.clone().get_required_value("metadata").change_context(errors::ConnectorError::MissingRequiredField { field_name: "metadata" })?.parse_value("metadata").change_context(errors::ConnectorError::RequestEncodingFailed)?;
        
        let merchant_ref = "Dummy Ref".to_string();
        let transaction_type = "void".to_string();
        let method = "credit_card".to_string();
        let amount = meta.amount;
        let currency_code = meta.currency;
        let transaction_tag = meta.tag;

        Ok(Self {
            merchant_ref,
            transaction_tag,
            transaction_type,
            method,
            amount,
            currency_code,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum VoidStatus {
    #[serde(rename = "approved")]
    Succeeded,
    #[serde(rename = "declined")]
    Failed,
    #[default]
    #[serde(rename = "not processed")]
    Processing,
}


impl From<VoidStatus> for enums::AttemptStatus {
    fn from(item: VoidStatus) -> Self {
        match item {
            VoidStatus::Succeeded => Self::Voided,
            VoidStatus::Failed => Self::VoidFailed,
            VoidStatus::Processing => Self::VoidInitiated,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeezyVoidResponse {
    #[serde(rename = "correlation_id")]
    pub correlation_id: String,
    #[serde(rename = "transaction_status")]
    pub transaction_status: VoidStatus,
    #[serde(rename = "validation_status")]
    pub validation_status: String,
    #[serde(rename = "transaction_type")]
    pub transaction_type: String,
    #[serde(rename = "transaction_id")]
    pub transaction_id: String,
    #[serde(rename = "transaction_tag")]
    pub transaction_tag: String,
    pub method: String,
    pub amount: String,
    pub currency: String,
    pub token: Token,
    #[serde(rename = "bank_resp_code")]
    pub bank_resp_code: String,
    #[serde(rename = "bank_message")]
    pub bank_message: String,
    #[serde(rename = "gateway_resp_code")]
    pub gateway_resp_code: String,
    #[serde(rename = "gateway_message")]
    pub gateway_message: String,
    #[serde(rename = "retrieval_ref_no")]
    pub retrieval_ref_no: String,
}

impl<F, T> 
    TryFrom<types::ResponseRouterData<F, PayeezyVoidResponse, T, types::PaymentsResponseData>,
    > for types::RouterData<F, T, types::PaymentsResponseData>
    {
        type Error = error_stack::Report<errors::ConnectorError>;
        fn try_from(
            item: types::ResponseRouterData<
                F,
                PayeezyVoidResponse,
                T,
                types::PaymentsResponseData,
            >,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                status: enums::AttemptStatus::from(item.response.transaction_status),
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                    resource_id: types::ResponseId::NoResponseId,
                    redirect: false,
                    redirection_data: None,
                    mandate_reference: None,
                    connector_metadata: Some(serde_json::to_value(PayeezyMeta{ 
                        tag: item.response.transaction_tag.to_string(),
                        amount: item.response.amount.to_string(),
                        currency: item.response.currency.to_string()
                    }).into_report().change_context(errors::ConnectorError::MissingRequiredField { field_name: ("metadata") })?, ),
                }),
                amount_captured: None,
                ..item.data
            })  
        }
    }


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeezyCaptureRequest {
    #[serde(rename = "merchant_ref")]
    pub merchant_ref: String,
    #[serde(rename = "transaction_tag")]
    pub transaction_tag: String,
    #[serde(rename = "transaction_type")]
    pub transaction_type: String,
    pub method: String,
    pub amount: String,
    #[serde(rename = "currency_code")]
    pub currency_code: String,
}


impl TryFrom<&types::PaymentsCaptureRouterData> for PayeezyCaptureRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_item: &types::PaymentsCaptureRouterData) -> Result<Self, Self::Error> {
        let merchant_ref = "Dummy Ref".to_string();
        
        let transaction_type = "capture".to_string();
        let method = "credit_card".to_string();
        let amount = _item.request.amount.to_string();
        let currency_code = _item.request.currency.to_string();
        let tag: PayeezyMeta = _item.request.connector_metadata.clone().get_required_value("metadata").change_context(errors::ConnectorError::MissingRequiredField { field_name: "metadata" })?.parse_value("metadata").change_context(errors::ConnectorError::RequestEncodingFailed)?;
        let transaction_tag = tag.tag;


        Ok(Self {
            merchant_ref,
            transaction_tag,
            transaction_type,
            method,
            amount,
            currency_code,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum CaptureStatus {
    #[serde(rename = "approved")]
    Succeeded,
    #[serde(rename = "declined")]
    Failed,
    #[default]
    #[serde(rename = "not processed")]
    Processing,
}


impl From<CaptureStatus> for enums::AttemptStatus {
    fn from(item: CaptureStatus) -> Self {
        match item {
            CaptureStatus::Succeeded => Self::Charged,
            CaptureStatus::Failed => Self::Failure,
            CaptureStatus::Processing => Self::Authorizing,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResponse {
    #[serde(rename = "correlation_id")]
    pub correlation_id: String,
    #[serde(rename = "transaction_status")]
    pub transaction_status: CaptureStatus,
    #[serde(rename = "validation_status")]
    pub validation_status: String,
    #[serde(rename = "transaction_type")]
    pub transaction_type: String,
    #[serde(rename = "transaction_id")]
    pub transaction_id: String,
    #[serde(rename = "transaction_tag")]
    pub transaction_tag: String,
    pub method: String,
    pub amount: String,
    pub currency: String,
    pub token: Token,
    #[serde(rename = "bank_resp_code")]
    pub bank_resp_code: String,
    #[serde(rename = "bank_message")]
    pub bank_message: String,
    #[serde(rename = "gateway_resp_code")]
    pub gateway_resp_code: String,
    #[serde(rename = "gateway_message")]
    pub gateway_message: String,
    #[serde(rename = "retrieval_ref_no")]
    pub retrieval_ref_no: String,
}

// impl TryFrom<types::PaymentsCaptureRouterData<>
//     for types::PaymentsCaptureData<>
//     {
//         type Error = error_stack::Report<errors::ParsingError>;
//         fn try_from(
//             _item: types::PaymentsCaptureResponseRouterData<CaptureResponse>,
//         )-> Result<Self, Self::Error> {
//             Ok(Self {
//                 amount_to_capture: _item.response.amount,
//                 currency: _item.response.currency,
//                 connector_transaction_id: _item.response.transaction_id,
//                 amount: _item.response.amount,
//             })
//         }
//     }

impl<F, T> 
    TryFrom<types::ResponseRouterData<F, CaptureResponse, T, types::PaymentsResponseData>,
    > for types::RouterData<F, T, types::PaymentsResponseData>
    {
        type Error = error_stack::Report<errors::ConnectorError>;
        fn try_from(
            item: types::ResponseRouterData<
                F,
                CaptureResponse,
                T,
                types::PaymentsResponseData,
            >,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                status: enums::AttemptStatus::from(item.response.transaction_status),
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                    resource_id: types::ResponseId::NoResponseId,
                    redirect: false,
                    redirection_data: None,
                    mandate_reference: None,
                    connector_metadata: Some(serde_json::to_value(PayeezyMeta{ 
                        tag: item.response.transaction_tag.to_string(),
                        amount: item.response.amount.to_string(),
                        currency: item.response.currency.to_string()
                    }).into_report().change_context(errors::ConnectorError::MissingRequiredField { field_name: ("metadata") })?, ),
                }),
                amount_captured: None,
                ..item.data
            })  
        }
    }



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeezyRefundRequest {
    #[serde(rename = "merchant_ref")]
    pub merchant_ref: String,
    #[serde(rename = "transaction_tag")]
    pub transaction_tag: String,
    #[serde(rename = "transaction_type")]
    pub transaction_type: String,
    pub method: String,
    pub amount: String,
    #[serde(rename = "currency_code")]
    pub currency_code: String,
}





impl<F> TryFrom<&types::RefundsRouterData<F>> for PayeezyRefundRequest {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: &types::RefundsRouterData<F>) -> Result<Self, Self::Error> {
        let merchant_ref = "Dummy red".to_string();
        let transaction_tag = _item.request.connector_transaction_id.to_string();
        let transaction_type = "refund".to_string();
        let method = "credit_card".to_string();
        let amount = _item.request.amount.to_string();
        let currency_code = _item.request.currency.to_string();

        Ok(Self {
            merchant_ref,
            transaction_tag,
            transaction_type,
            method,
            amount,
            currency_code,
        })
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    #[serde(rename = "approved")]
    Succeeded,
    #[serde(rename = "declined")]
    Failed,
    #[default]
    #[serde(rename = "not processed")]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundResponse {
    #[serde(rename = "correlation_id")]
    pub correlation_id: String,
    #[serde(rename = "transaction_status")]
    pub transaction_status: RefundStatus,
    #[serde(rename = "validation_status")]
    pub validation_status: String,
    #[serde(rename = "transaction_type")]
    pub transaction_type: String,
    #[serde(rename = "transaction_id")]
    pub transaction_id: String,
    #[serde(rename = "transaction_tag")]
    pub transaction_tag: String,
    pub method: String,
    pub amount: String,
    pub currency: String,
    pub token: Token,
    #[serde(rename = "bank_resp_code")]
    pub bank_resp_code: String,
    #[serde(rename = "bank_message")]
    pub bank_message: String,
    #[serde(rename = "gateway_resp_code")]
    pub gateway_resp_code: String,
    #[serde(rename = "gateway_message")]
    pub gateway_message: String,
    #[serde(rename = "retrieval_ref_no")]
    pub retrieval_ref_no: String,
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        _item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: _item.response.transaction_id,
                refund_status: enums::RefundStatus::from(_item.response.transaction_status),
            }),
            .._item.data
        })
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>>
    for types::RefundsRouterData<api::RSync>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        _item: types::RefundsResponseRouterData<api::RSync, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: _item.response.transaction_id,
                refund_status: enums::RefundStatus::from(_item.response.transaction_status),
            }),
            .._item.data
        })
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct PayeezyErrorResponse {}
