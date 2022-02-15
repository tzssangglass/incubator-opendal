// Copyright 2022 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use futures::lock::Mutex;
use opendal::ops::OpDelete;
use opendal::services::fs;
use opendal::Accessor;
use opendal::Layer;
use opendal::Operator;

struct Test {
    #[allow(dead_code)]
    inner: Option<Arc<dyn Accessor>>,
    deleted: Arc<Mutex<bool>>,
}

impl Layer for &Test {
    fn layer(&self, inner: Arc<dyn Accessor>) -> Arc<dyn Accessor> {
        Arc::new(Test {
            inner: Some(inner.clone()),
            deleted: self.deleted.clone(),
        })
    }
}

#[async_trait::async_trait]
impl Accessor for Test {
    async fn delete(&self, _args: &OpDelete) -> opendal::error::Result<()> {
        let mut x = self.deleted.lock().await;
        *x = true;

        assert!(self.inner.is_some());

        // We will not call anything here to test the layer.
        Ok(())
    }
}

#[tokio::test]
async fn test_layer() {
    let test = Test {
        inner: None,
        deleted: Arc::new(Mutex::new(false)),
    };

    let op = Operator::new(fs::Backend::build().finish().await.unwrap()).layer(&test);

    op.delete("xxxxx").run().await.unwrap();

    assert!(*test.deleted.clone().lock().await);
}