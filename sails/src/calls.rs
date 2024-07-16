use crate::{
    errors::{Error, Result, RtlError},
    prelude::*,
};
use core::{future::Future, marker::PhantomData};

pub trait Action<TArgs> {
    fn with_value(self, value: ValueUnit) -> Self;

    fn with_args(self, args: TArgs) -> Self;

    fn value(&self) -> ValueUnit;

    fn args(&self) -> &TArgs;
}

#[allow(async_fn_in_trait)]
pub trait Call<TArgs, TReply>: Action<TArgs> {
    async fn send(self, target: ActorId) -> Result<impl Reply<T = TReply>>;

    async fn send_recv(self, target: ActorId) -> Result<TReply>
    where
        Self: Sized,
    {
        self.send(target).await?.recv().await
    }
}

#[allow(async_fn_in_trait)]
pub trait Activation<TArgs>: Action<TArgs> {
    async fn send(self, code_id: CodeId, salt: impl AsRef<[u8]>)
        -> Result<impl Reply<T = ActorId>>;

    async fn send_recv(self, code_id: CodeId, salt: impl AsRef<[u8]>) -> Result<ActorId>
    where
        Self: Sized,
    {
        self.send(code_id, salt).await?.recv().await
    }
}

#[allow(async_fn_in_trait)]
pub trait Query<TArgs, TReply>: Action<TArgs> {
    async fn recv(self, target: ActorId) -> Result<TReply>;
}

#[allow(async_fn_in_trait)]
pub trait Reply {
    type T;
    async fn recv(self) -> Result<Self::T>;
}

pub struct CallTicket<TReplyFuture, TActionIo> {
    reply_future: TReplyFuture,
    _io: PhantomData<TActionIo>,
}

impl<TReplyFuture, TActionIo> CallTicket<TReplyFuture, TActionIo> {
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _io: PhantomData,
        }
    }
}

impl<TReplyFuture, TActionIo> Reply for CallTicket<TReplyFuture, TActionIo>
where
    TReplyFuture: Future<Output = Result<Vec<u8>>>,
    TActionIo: ActionIo,
{
    type T = TActionIo::Reply;

    async fn recv(self) -> Result<Self::T> {
        let reply_bytes = self.reply_future.await?;
        TActionIo::decode_reply(reply_bytes)
    }
}

pub struct ActivationTicket<TReplyFuture, TActionIo> {
    reply_future: TReplyFuture,
    _io: PhantomData<TActionIo>,
}

impl<TReplyFuture, TActionIo> ActivationTicket<TReplyFuture, TActionIo> {
    pub(crate) fn new(reply_future: TReplyFuture) -> Self {
        Self {
            reply_future,
            _io: PhantomData,
        }
    }
}

impl<TReplyFuture, TActionIo> Reply for ActivationTicket<TReplyFuture, TActionIo>
where
    TReplyFuture: Future<Output = Result<(ActorId, Vec<u8>)>>,
    TActionIo: ActionIo<Reply = ()>,
{
    type T = ActorId;

    async fn recv(self) -> Result<Self::T> {
        let (actor_id, payload) = self.reply_future.await?;
        TActionIo::decode_reply(payload)?;
        Ok(actor_id)
    }
}

#[allow(async_fn_in_trait)]
pub trait Remoting<TArgs> {
    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: TArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>>;

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: TArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>>;

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        value: ValueUnit,
        args: TArgs,
    ) -> Result<Vec<u8>>;
}

pub struct RemotingAction<TRemoting, TArgs, TActionIo: ActionIo> {
    remoting: TRemoting,
    params: TActionIo::Params,
    value: ValueUnit,
    args: TArgs,
}

impl<TRemoting, TArgs, TActionIo: ActionIo> RemotingAction<TRemoting, TArgs, TActionIo>
where
    TArgs: Default,
{
    pub fn new(remoting: TRemoting, params: TActionIo::Params) -> Self {
        Self {
            remoting,
            params,
            value: Default::default(),
            args: Default::default(),
        }
    }
}

impl<TRemoting, TArgs, TActionIo: ActionIo> Action<TArgs>
    for RemotingAction<TRemoting, TArgs, TActionIo>
{
    fn with_value(self, value: ValueUnit) -> Self {
        Self { value, ..self }
    }

    fn with_args(self, args: TArgs) -> Self {
        Self { args, ..self }
    }

    fn value(&self) -> ValueUnit {
        self.value
    }

    fn args(&self) -> &TArgs {
        &self.args
    }
}

impl<TRemoting, TArgs, TActionIo> Call<TArgs, TActionIo::Reply>
    for RemotingAction<TRemoting, TArgs, TActionIo>
where
    TRemoting: Remoting<TArgs>,
    TActionIo: ActionIo,
{
    async fn send(self, target: ActorId) -> Result<impl Reply<T = TActionIo::Reply>> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_future = self
            .remoting
            .message(target, payload, self.value, self.args)
            .await?;
        Ok(CallTicket::<_, TActionIo>::new(reply_future))
    }
}

impl<TRemoting, TArgs, TActionIo> Activation<TArgs> for RemotingAction<TRemoting, TArgs, TActionIo>
where
    TRemoting: Remoting<TArgs>,
    TActionIo: ActionIo<Reply = ()>,
{
    async fn send(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
    ) -> Result<impl Reply<T = ActorId>> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_future = self
            .remoting
            .activate(code_id, salt, payload, self.value, self.args)
            .await?;
        Ok(ActivationTicket::<_, TActionIo>::new(reply_future))
    }
}

impl<TRemoting, TArgs, TActionIo> Query<TArgs, TActionIo::Reply>
    for RemotingAction<TRemoting, TArgs, TActionIo>
where
    TRemoting: Remoting<TArgs>,
    TActionIo: ActionIo,
{
    async fn recv(self, target: ActorId) -> Result<TActionIo::Reply> {
        let payload = TActionIo::encode_call(&self.params);
        let reply_bytes = self
            .remoting
            .query(target, payload, self.value, self.args)
            .await?;
        TActionIo::decode_reply(reply_bytes)
    }
}

pub trait ActionIo {
    const ROUTE: &'static [u8];
    type Params: Encode;
    type Reply: Decode;

    fn encode_call(value: &Self::Params) -> Vec<u8> {
        let mut result = Vec::with_capacity(Self::ROUTE.len() + value.encoded_size());
        result.extend_from_slice(Self::ROUTE);
        value.encode_to(&mut result);
        result
    }

    fn decode_reply(payload: impl AsRef<[u8]>) -> Result<Self::Reply> {
        let mut value = payload.as_ref();
        if !value.starts_with(Self::ROUTE) {
            return Err(Error::Rtl(RtlError::ReplyPrefixMismatches));
        }
        value = &value[Self::ROUTE.len()..];
        Decode::decode(&mut value).map_err(Error::Codec)
    }
}