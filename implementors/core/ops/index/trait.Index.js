(function() {var implementors = {};
implementors["image"] = ["impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.Rgb.html\" title=\"struct image::Rgb\">Rgb</a>&lt;T&gt;","impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.Luma.html\" title=\"struct image::Luma\">Luma</a>&lt;T&gt;","impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.Rgba.html\" title=\"struct image::Rgba\">Rgba</a>&lt;T&gt;","impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.LumaA.html\" title=\"struct image::LumaA\">LumaA</a>&lt;T&gt;","impl&lt;P, Container&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u32.html\">u32</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u32.html\">u32</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"image/struct.ImageBuffer.html\" title=\"struct image::ImageBuffer\">ImageBuffer</a>&lt;P, Container&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;P: <a class=\"trait\" href=\"image/trait.Pixel.html\" title=\"trait image::Pixel\">Pixel</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;P::<a class=\"type\" href=\"image/trait.Pixel.html#associatedtype.Subpixel\" title=\"type image::Pixel::Subpixel\">Subpixel</a>: 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;Container: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a>&lt;Target = <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">[</a>P::<a class=\"type\" href=\"image/trait.Pixel.html#associatedtype.Subpixel\" title=\"type image::Pixel::Subpixel\">Subpixel</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">]</a>&gt;,&nbsp;</span>",];
implementors["nalgebra"] = ["impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/core/trait.Scalar.html\" title=\"trait nalgebra::core::Scalar\">Scalar</a>, R:&nbsp;<a class=\"trait\" href=\"nalgebra/core/dimension/trait.Dim.html\" title=\"trait nalgebra::core::dimension::Dim\">Dim</a>, C:&nbsp;<a class=\"trait\" href=\"nalgebra/core/dimension/trait.Dim.html\" title=\"trait nalgebra::core::dimension::Dim\">Dim</a>, S:&nbsp;<a class=\"trait\" href=\"nalgebra/core/storage/trait.Storage.html\" title=\"trait nalgebra::core::storage::Storage\">Storage</a>&lt;N, R, C&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"nalgebra/core/struct.Matrix.html\" title=\"struct nalgebra::core::Matrix\">Matrix</a>&lt;N, R, C, S&gt;","impl&lt;N, R:&nbsp;<a class=\"trait\" href=\"nalgebra/core/dimension/trait.Dim.html\" title=\"trait nalgebra::core::dimension::Dim\">Dim</a>, C:&nbsp;<a class=\"trait\" href=\"nalgebra/core/dimension/trait.Dim.html\" title=\"trait nalgebra::core::dimension::Dim\">Dim</a>, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"nalgebra/core/struct.Matrix.html\" title=\"struct nalgebra::core::Matrix\">Matrix</a>&lt;N, R, C, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N: <a class=\"trait\" href=\"nalgebra/core/trait.Scalar.html\" title=\"trait nalgebra::core::Scalar\">Scalar</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"nalgebra/core/storage/trait.Storage.html\" title=\"trait nalgebra::core::storage::Storage\">Storage</a>&lt;N, R, C&gt;,&nbsp;</span>","impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/core/trait.Scalar.html\" title=\"trait nalgebra::core::Scalar\">Scalar</a>, D:&nbsp;<a class=\"trait\" href=\"nalgebra/core/dimension/trait.DimName.html\" title=\"trait nalgebra::core::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.Point.html\" title=\"struct nalgebra::geometry::Point\">Point</a>&lt;N, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/core/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::core::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/core/allocator/trait.Allocator.html\" title=\"trait nalgebra::core::allocator::Allocator\">Allocator</a>&lt;N, D&gt;,&nbsp;</span>","impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/core/trait.Scalar.html\" title=\"trait nalgebra::core::Scalar\">Scalar</a>, D:&nbsp;<a class=\"trait\" href=\"nalgebra/core/dimension/trait.DimName.html\" title=\"trait nalgebra::core::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.Rotation.html\" title=\"struct nalgebra::geometry::Rotation\">Rotation</a>&lt;N, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/core/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::core::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/core/allocator/trait.Allocator.html\" title=\"trait nalgebra::core::allocator::Allocator\">Allocator</a>&lt;N, D, D&gt;,&nbsp;</span>","impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.Real.html\" title=\"trait nalgebra::Real\">Real</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.Quaternion.html\" title=\"struct nalgebra::geometry::Quaternion\">Quaternion</a>&lt;N&gt;","impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.Real.html\" title=\"trait nalgebra::Real\">Real</a>, D, C:&nbsp;<a class=\"trait\" href=\"nalgebra/geometry/trait.TCategory.html\" title=\"trait nalgebra::geometry::TCategory\">TCategory</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.Transform.html\" title=\"struct nalgebra::geometry::Transform\">Transform</a>&lt;N, D, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;D: <a class=\"trait\" href=\"nalgebra/core/dimension/trait.DimName.html\" title=\"trait nalgebra::core::dimension::DimName\">DimName</a> + <a class=\"trait\" href=\"nalgebra/core/dimension/trait.DimNameAdd.html\" title=\"trait nalgebra::core::dimension::DimNameAdd\">DimNameAdd</a>&lt;<a class=\"struct\" href=\"nalgebra/core/dimension/struct.U1.html\" title=\"struct nalgebra::core::dimension::U1\">U1</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/core/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::core::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/core/allocator/trait.Allocator.html\" title=\"trait nalgebra::core::allocator::Allocator\">Allocator</a>&lt;N, <a class=\"type\" href=\"nalgebra/core/dimension/type.DimNameSum.html\" title=\"type nalgebra::core::dimension::DimNameSum\">DimNameSum</a>&lt;D, <a class=\"struct\" href=\"nalgebra/core/dimension/struct.U1.html\" title=\"struct nalgebra::core::dimension::U1\">U1</a>&gt;, <a class=\"type\" href=\"nalgebra/core/dimension/type.DimNameSum.html\" title=\"type nalgebra::core::dimension::DimNameSum\">DimNameSum</a>&lt;D, <a class=\"struct\" href=\"nalgebra/core/dimension/struct.U1.html\" title=\"struct nalgebra::core::dimension::U1\">U1</a>&gt;&gt;,&nbsp;</span>",];
implementors["rustc_serialize"] = ["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;&amp;'a <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt; for <a class=\"enum\" href=\"rustc_serialize/json/enum.Json.html\" title=\"enum rustc_serialize::json::Json\">Json</a>","impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"enum\" href=\"rustc_serialize/json/enum.Json.html\" title=\"enum rustc_serialize::json::Json\">Json</a>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
