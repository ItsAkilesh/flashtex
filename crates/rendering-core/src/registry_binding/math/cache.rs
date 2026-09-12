//! Borrowed immutable MATH query cache. Source-bound frames are reconstructed;
//! source revisions and renderer lease epochs are never cached away.
use super::*;
use super::{assembly::*, device::*, kern::*};
use crate::mixed::MixedLimits;
use flashtex_font_resources::math_cache::{
    CacheError, Limits, MathQueryCache, Query, QueryError, Stats, Value,
};
#[derive(Debug)]
pub enum CachedMathError {
    Metric(MathConsumerError),
    Cache(CacheError),
    Unavailable(Arc<std::result::Result<Value, QueryError>>),
    Assembly(AssemblyError),
    Device(DeviceConsumerError),
}
impl From<MathConsumerError> for CachedMathError {
    fn from(e: MathConsumerError) -> Self {
        Self::Metric(e)
    }
}
impl From<ValidationError> for CachedMathError {
    fn from(e: ValidationError) -> Self {
        Self::Metric(e.into())
    }
}
pub struct RegistryMathCache<'a> {
    lease: &'a MathLease,
    cache: MathQueryCache<'a>,
}
impl RegistryRenderer {
    pub fn math_cache<'a>(
        &self,
        lease: &'a MathLease,
        limits: Limits,
    ) -> std::result::Result<RegistryMathCache<'a>, CachedMathError> {
        self.current(&lease.render)
            .map_err(MathConsumerError::from)?;
        Ok(RegistryMathCache {
            lease,
            cache: MathQueryCache::new(&lease.font, limits).map_err(CachedMathError::Cache)?,
        })
    }
}
impl RegistryMathCache<'_> {
    pub fn stats(&self) -> Stats {
        self.cache.stats()
    }
    pub fn identity(&self) -> &MathIdentity {
        self.cache.identity()
    }
    fn query(
        &mut self,
        renderer: &RegistryRenderer,
        query: Query,
    ) -> std::result::Result<Value, CachedMathError> {
        renderer
            .current(&self.lease.render)
            .map_err(MathConsumerError::from)?;
        let reply = self
            .cache
            .query(self.lease.identity(), query)
            .map_err(CachedMathError::Cache)?;
        match reply.outcome.as_ref() {
            Ok(value) => Ok(value.clone()),
            Err(_) => Err(CachedMathError::Unavailable(reply.outcome)),
        }
    }
    pub fn assembly(
        &mut self,
        renderer: &RegistryRenderer,
        request: AssemblyRequest<'_>,
        limits: MixedLimits,
    ) -> std::result::Result<MathAssemblyFrame, CachedMathError> {
        let value = self.query(
            renderer,
            Query::Fit {
                glyph_id: request.original_gid,
                direction: request.direction,
                target: request.target,
                strategy: request.strategy,
                limits: request.fit_limits,
            },
        )?;
        let Value::Fit(fit) = value else {
            return Err(ValidationError("cached MATH fit kind".into()).into());
        };
        renderer
            .math_assembly_with_fit(self.lease, request, limits, Some(fit))
            .map_err(CachedMathError::Assembly)
    }
    pub fn devices(
        &mut self,
        renderer: &RegistryRenderer,
        query: MathQuery<'_>,
        scale: PixelScale,
        requests: &[DeviceQuery],
    ) -> std::result::Result<MathDeviceSnapshot, CachedMathError> {
        if requests.len() > 256 {
            return Err(MathConsumerError::Budget.into());
        }
        let mut values = Vec::with_capacity(requests.len());
        for &request in requests {
            let q = match request {
                DeviceQuery::Constant {
                    record, context, ..
                } => Query::Constant { record, context },
                DeviceQuery::Glyph {
                    original_gid,
                    kind,
                    context,
                    ..
                } => Query::Glyph {
                    glyph_id: original_gid,
                    kind,
                    context,
                },
                DeviceQuery::Kern {
                    original_gid,
                    corner,
                    height,
                    context,
                } => Query::Kern {
                    glyph_id: original_gid,
                    corner,
                    height,
                    context,
                },
            };
            values.push(self.query(renderer, q)?);
        }
        renderer
            .math_devices_with_values(self.lease, query, scale, requests, Some(&values))
            .map_err(CachedMathError::Device)
    }
    pub fn kerns(
        &mut self,
        renderer: &RegistryRenderer,
        query: MathQuery<'_>,
        requests: &[KernQuery],
    ) -> std::result::Result<MathKernSnapshot, CachedMathError> {
        if requests.len() > 256 {
            return Err(MathConsumerError::Budget.into());
        }
        let mut values = Vec::with_capacity(requests.len());
        for &request in requests {
            values.push(self.query(
                renderer,
                Query::UnhintedKern {
                    glyph_id: request.original_gid,
                    corner: request.corner,
                    height: request.height,
                },
            )?);
        }
        renderer
            .math_kerns_with_values(self.lease, query, requests, Some(&values))
            .map_err(CachedMathError::Metric)
    }
}
