## 基于多线程加速的 STEP 标准并行解析算法

毕玉莉 1,2) , 刘嘉政 1) , 杨义军 1)* , 曾薇 3) , 雍俊海 4)

1) ( 西安交通大学计算机科学与技术学院 西安 710049)

2) ( 中国船舶集团海舟系统技术有限公司 上海 200011)

3) ( 西安交通大学数学与统计学院 西安 710049)

4) ( 清华大学软件学院 北京 100084)

(yangyijun@xjtu.edu.cn)

摘 要 : 当前 CAD 软件加载大型 STEP 中性文件时速度慢 , 其效率瓶颈关键在于其解析算法 . 针对基于单线程的中 性标准解析算法已经远远不能满足当前 CAD 应用需求的问题 , 提出了一种基于多线程的分层并行数据解析算法 , 利 用当前主流 CPU 都采用的多核并行架构 , 大幅提升了 STEP 文件解析效率 . 根据模型体中数据的分层特性 , 算法利用 线程池对各层数据解析进行并行加速 . 文中算法已经在开源几何引擎 OpenGE 中实现 . 与 OpenCASCADE 的对比实 验验证了文中算法的有效性 . 实验结果表明 , 算法的并行解析程序运行速度相比串行解析程序运行速度提升至原算 法的 2 倍 , 比 OpenCASCADE 解析算法快 50%.

关键词 :

STEP 标准 ;

数据解析 ;

多线程 ;

并行处理算法

中图法分类号 :

TP391.41

DOI:

10.3724/SP.J.1089.2023.2023-00037

## A Parallel Algorithm to Parse STEP Files Based on Multiple Threads

Bi Yuli 1,2) , Liu Jiazheng 1) , Yang Yijun 1)* , Zeng Wei 3) , and Yong Junhai 4)

1) ( School of Computer Science and Technology, Xi'an Jiaotong University, Xi'an 710049)

2) ( China State Shipbuilding Haizhou System and Technology Corporation, Shanghai 200011)

3) ( School of Mathematics and Statistics, Xi'an Jiaotong University, Xi'an 710049)

4) ( School of Software, Tsinghua University, Beijing 100084)

Abstract: Current CAD software is slow to load large STEP neutral files, and its efficiency bottleneck lies in its analytic algorithm. The parsing algorithm based on single thread cannot meet the requirements of CAD application.  This  paper  proposes  a  hierarchical  parallel  data  parsing  algorithm  based  on  multithreading, which makes use of the multi-core parallel architecture adopted by current mainstream CPUs, and greatly improves  the  efficiency  of  STEP  file  parsing.  According  to  the  hierarchical  characteristics  of  data  in  the model, the parallel acceleration of data parsing at each layer is carried out by using thread pool. The algorithm in this paper has been implemented in open source geometry engine OpenGE. In this paper, comparison between our algorithm and OpenCASCADE is performed to verify the running effects of the parallel algorithms. The experimental results show that the running speed of the parallel parsing program is about one times faster than that of the serial parsing program based on single thread and about one and a half times faster than that of OpenCASCADE.

收稿日期 :  2023-04-20; 修回日期 :  2023-08-29. 基金项目 : 国家重点研发计划 (2020YFB1708900); 国家自然科学基金 (62021002, 12090021,  61872224). 毕玉莉 (1997-), 女 , 硕士研究生 , 主要研究方向为计算机辅助设计与计算机图形学 ; 刘嘉政 (2001-), 男 , 在 校学生 , 主要研究方向为计算机辅助设计与计算机图形学 ; 杨义军 (1979-), 男 , 博士 , 教授 , 博士生导师 ,  CCF 会员 , 论文通信作者 , 主 要研究方向为计算机辅助设计与计算机图形学 ; 曾薇 (1980 -), 女 , 博士 , 教授 , 博士生导师 ,  CCF 会员 , 主要研究方向为数字几何处理 ; 雍俊海 (1973 -), 男 , 博士 , 教授 , 博士生导师 , CCF 会员 , 主要研究方向为计算机辅助设计与计算机图形学 .

Key words: STEP standard; data parsing; multi-thread technique; parallel processing algorithm

20 世纪 70 年代末 , 伴随着工业领域的快速发 展 ,  CAX 系统的应用也得到了大力推广 . 然而 , 各 系统之间数据格式不统一 , 系统不兼容的问题使得 各系统在进行信息交换时出现信息冗余、 错误或缺 失等 , 极大地限制了工业界的发展 . 为解决上述问 题 , 人们提出了多种数据标准来实现产品数据信 息之间的交换与表达 . 其中影响较大的有 IGES 标 准和 SET 标准等多种标准 . 这些数据标准在不同 国家、不同工业领域中都得到了广泛的应用 , 但是 这些标准中仍然存在着各种不足 , 并不能满足工 业生产过程中整个产品生命周期的数据交换要求 .

随着各国数据交换标准的发展 , 大家逐渐意 识到必须通过一种国家标准来规范不同 CAX 系统 间的数据交换 . 因此 , 国际标准化组织 (ISO) 对这 些标准进行了统一 , 并于 1983 年制定了一个技术 产品数据全方面的国家标准 --产品数据交换标 准 (standard for the exchange of product model data, STEP).  STEP 标准的目标是提供一种不依赖具体 系统的中性机制 , 以描述一个在产品生存期内具 有完整性与集成性的计算机化的产品模型所需的 全部信息 , 同时可以保证在整个 CAD/CAX 信息交 换过程中数据的一致性和完整性 .

在过去 20 多年里 , 有不少学者对 STEP 标准 进行了相关研究 . 文献 [1-8] 主要对 STEP 标准的产 生、体系结构等做了简要概述 . 文献 [9-13] 主要介 绍了 STEP 中性文件的几种具体实施过程 , 如张思 荣等 [9] 介绍了中性文件的实施过程和方法 , 以及二 维工程图中的主要实体在 STEP 中性文件中的描 述 , 并给出了具体示例 . 邱长华等 [10] 提出了将数 据从 CAD 系统导出到 STEP 文件的前置处理器的 结构 . 朱大培等 [11] 提出了 CAD 造型系统到其自行 研制的可视化系统的后置处理器 . 文献 [14-17] 介 绍了基于 STEP 标准的 CAX 系统间的集成技术 , 如段广洪等 [14] 阐述了 CAD/CAPP/CAM 集成系统 的集成原理和接口设计 .

本文采用并行数据解析程序来实现系统间的 数据转换 , 解决了现有的数据解析算法数据转换 速度慢的问题 , 有效地提升了文件数据读取效率 . 本文算法已经在开源几何引擎 OpenGE 中实现 , 其 源代码会在国家重点研发项目结题后进行公开 .

## 1    STEP 文件读取算法实现

对于一个具体的 CAD 应用系统 , 当采用 STEP

文件进行产品数据交换时 , 必须设计 STEP 文件的 前置处理器和后置处理器 , 即分别从 CAX 系统到 STEP 文件的处理接口 , 以及从 STEP 文件到另一 个 CAX 系统的处理接口 , 如图 1 所示 .

图 1 不同软件间的数据交换

<!-- image -->

本文提出了一种并行数据解析算法 , 实现了 后置处理器的功能 , 将 STEP 中性文件读入几何引 擎 , 前置处理器功能可以采用本文类似的并行策 略 . 后置处理器主要包括 5 个模块 , 分别是输入模 块、数据分析模块、数据转换模块、数据合法性检 查以及输出模块 , 具体流程如图 2 所示 .

图 2    STEP 中性文件读取流程图

<!-- image -->

## 1.1 输入和输出模块

输入模块主要提供了一系列接口 , 处理器通 过这些接口可以读入 CAD 模型的多种格式文件 , 从而实现 CAD 工业软件与其他 CAD/CAM 软件之 间的数据交换 . 本文算法通过 STEP 中性文件实现 数据交换 , 其中输入模块通过扫描 STEP 中性文件 将数据读取至内存 .

输出模块输出经过处理转换后得到的几何引 擎中的实体类对象 . 通过实体类对象之间的相互 索引以及实体类对象包含的属性值 , 得到三维模 型的所有几何实体数据以及数据间的拓扑关系 .

## 1.2 数据分析模块

数据分析模块针对输入的中性文件数据进行 词法和语法分析 , 其只处理产品几何和拓扑数据 , 暂不处理无关内容 . 无关内容指的是中性文件中 头部段中的内容和数据段中部分实体实例数据 ( 非 几何和拓扑类实例数据 ), 即在处理数据时仅处理 文件中数据段内的几何实例数据和拓扑实例数据 .

对输入的数据执行分析算法 , 分析算法流程 图如图 3 所示 .

图 3 数据分析模块流程图

<!-- image -->

数据分析算法的各个步骤如下 .

Step1. 读取一行数据 , 判断是否位于数据段 , 如果 是 , 则继续向下执行 ; 否则 , 暂不做任何处理 , 继续读 取下一行 .

Step2. 判断数据包含信息是否完整 .

通常 , 一个实体实例数据只占一行 , 但当实例数据 包含信息过多时 , 也会出现占据多行的情形 . 因此 , 在 对数据处理前要判断数据是否完整 , 不完整时需对多行 数据进行拼接 , 使数据完整 .

Step3. 对数据进行语法分析 . 去掉数据中包含的 多余字符 , 如去掉数据中多余的双引号、空格以及换行 符号等 , 方便后续提取实例信息 .

Step4. 提取实例数据信息并保存 . 文件中所有几 何和拓扑数据的格式类似 , 且都包含实例标识符、实例 类型以及实例包含的属性值等内容 . 对每一行数据都提 取以上内容 , 并将数据的地址存放到一个一维数组中 , 数据在数组中存放的顺序和数据的标识符一致 .

## 1.3 数据转换模块

数据转换模块的主要作用是将经过数据分析 模块处理后的数据转换成 CAD 几何引擎内置数据 结构 , 是整个框架中的核心部分 .

数据转换模块主要包含数据映射、 语义分析和 数据转换 3 部分功能 .

## 1.3.1 数据映射

数据映射建立 STEP 标准几何和拓扑数据到

CAD 几何引擎内部数据结构之间的映射 , 找到两 者定义的实体之间的对应关系 .

## (1) EXPRESS 语言到 C++ 语言的映射

STEP 标准采用拓展边界表示法来描述模型的 几何和拓扑数据信息 . 针对几何和拓扑信息中的 每一类概念 , STEP 标准均通过 EXPRESS 语言给出 了详细的描述以及定义 .  EXPRESS 语言不同于一 般的程序设计语言 , 是一种信息建模语言 , 它结合 了现有的各种程序设计语言的优点 , 既有利于计 算机的处理 , 也便于使用者理解 . 它具有面向对象 的特点 , 定义的模型数据的数据结构也与 C++ 中 定义的类的结构类似 , 同样也具有继承关系 . 因 此 , 本文通过 C++ 语言来实现 EXPRESS 语言定义 的类结构 , 以方便计算机编程 .

以 STEP 标准中定义的几何类型椭圆为例 , 在 STEP 标准中椭圆的 EXPRESS 表示如下 .

## ENTITY ellipse

```
SUBTYPE OF (conic); semi_axis_1: positive_length_measure; semi_axis_2: positive_length_measure; END_ENTITY; 映射成对应的 C++ 类的表示如下 . class ellipse:public Conic { private: double: semi_axis_1; double: semi_axis_1; }
```

(2) STEP 标准数据结构到 CAD 几何引擎数据 结构的映射

经过 EXPRESS 语言到 C++ 语言的映射 , 得到 STEP 数据结构在内存中的存储形式 , 即每一类数 据结构都可以转换成一个对应的 C++ 类 . 因此 , STEP 数据结构到几何引擎数据结构的映射可以转 换成 STEP 标准数据结构对应的 C++ 类与几何引擎 中数据结构类之间的映射 .

STEP 标准以及几何引擎都对实体模型的几何 和拓扑数据信息进行了详细描述及定义 , 但是两者 对于同一类数据结构的描述可能会存在差异 . 例如 , 针对有向边的概念 ,  STEP 标准采用 ORIENTED\_ EDGE 类对其进行描述 , 而在几何引擎里面有向边 被定义为 COEDGE; 而对于椭圆的两者都用 ELLIPSE 来描述 . 因此 , 在数据进行转换前 , 需要 先根据数据实例所属的 STEP 实体类型找到几何引 擎中与之对应的实体类型 . 目前 STEP 标准拓扑类 和几何引擎拓扑类之间的映射关系如表 1 所示 , 几 何类之间的映射关系如表 2( 表中展示的仅是具体的

几何类 , 抽象类未在其中 ) 所示 , 本文几何和拓扑表 示以开源几何引擎 OpenGE 为例进行叙述 .

表 1 拓扑类映射关系

| STEP 标准拓扑类         | 拓扑类        |
|--------------------|------------|
| SOLID_MODEL        | BODY       |
| CONNECTED_FACE_SET | SHELL      |
| FACE               | FACE       |
| FACE_BOUND         | FACE_BOUND |
| LOOP               | LOOP       |
| PATH               |            |
| ORIECTED_EDGE      | COEDGE     |
| EDGE               | EDGE       |
| VERTEX             | VERTEX     |

表 2 几何类映射关系

| STEP 标准几何类                   | 几何类                    |
|------------------------------|------------------------|
| CARTESIAN_POINT              | POINT3D                |
| DIRECTION                    | DIRECTION3D            |
| VECTOR                       | VECTOR3D               |
| AXIS2_PLACEMENT_3D           | ENTITY_UCS             |
| LINE                         | LINE3D                 |
| CIRCLE                       | CIRCLE3D               |
| ELLIPSE                      | ELLIPSE3D              |
| HPERBOLA                     | HPERBOLA3D             |
| B_SPLINE_CURVE_WITH_KNOTS    | B_SPLINE_CURVE3D       |
| RATIONAL_B_SPLINE_CURVE      | B_SPLINE_CURVE3D       |
| PLANE                        | PLANE3D                |
| CYLINDRICAL_SURFACE          | CYLINDRICAL_ SURFACE3D |
| CONICAL_SURFACE              | CONICAL_SURFACE3D      |
| SPHERICAL_SURFACE            | SPHERICAL_SURFACE3D    |
| TOROIDAL_SURFACE             | TOROIDAL_SURFACE3D     |
| B_SPLINE_SURFACE_ WITH_KNOTS | B_SPLINE_SURFACE3D     |
| RATIONAL_B_SPLINE_SURFACE    | B_SPLINE_SURFACE3D     |

## 1.3.2 语义分析

除几何引擎实体类和 STEP 标准实体类之间 的映射之外 , 2 个类里面的属性类型也会存在差异 . 因此 , 在实现实体类型的映射后 , 还需要进一步对 参数进行分析 , 实现两者之间属性参数的转换 .

例如 , 文件中某一个数据的实例类型是一个 椭圆 , 它对应的几何引擎里的数据结构也是椭圆 . 在 STEP 标准中 , 椭圆通过一个局部坐标系、主轴 长度和短轴长度来定义一个椭圆 . 而在几何引擎 里 , 椭圆通过中心点位置、主轴方向、主轴长度、 所在平面法矢方向和椭圆率 (ratio) 来表示 . 因此 ,

在数据转换时需要对属性参数进行语义分析 , 根 据 STEP 标准中椭圆的属性信息计算得到几何引擎 中椭圆所需的属性信息 . 属性类型差异主要体现 在几何数据类型中 , 表 3 和表 4 分别列出了自主研 发几何数据类型所包含的属性以及 STEP 标准几何 引擎数据结构几何类型所包含的属性 .

表 3 自主研发几何引擎几何类属性

| 几何类                    | 包含属性                                                                                  |
|------------------------|---------------------------------------------------------------------------------------|
| POINT3D                | 3 个坐标值 x , y , z                                                                      |
| DIRECTION3D            | 3 个坐标值 x , y , z                                                                      |
| VECTOR3D               | DIRECTION3D 、长度                                                                       |
| ENTITY_UCS             | POINT3D 原点、 局部坐标系的 x 轴、 y 轴以及 z 轴                                                     |
| LINE3D                 | POINT3D 原点、 DIRECTION3D 方向                                                            |
| CIRCLE3D               | ENTITY_UCS 坐标系半径                                                                      |
| ELLIPSE3D              | 中心点位置、主轴方向、主轴长 度、所在平面法矢方向和椭圆率                                                         |
| B_SPLINE_CURVE3D       | int 次数 p 控制点 vector<POINT 3D> 节点数组 vector<double> 节点重复度 vector<int> 权重 vector<double> |
| PLANE3D                | POINT3D 三维点 DIRECTION3D z 轴方向                                                         |
| LINE3D                 | POINT3D 原点、 DIRECTION3D 方向                                                            |
| CYLINDRICAL_SURFAC E3D | ENTITY_UCS 坐标系 double 值 r 底面圆半径                                                       |
| CONICAL_SURFACE3D      | ENTITY_UCS 坐标系 double 值 c                                                             |
| SPHERICAL_SURFACE3D    | ENTITY_UCS 坐标系 double 值 r 表示球半径                                                       |
| TOROIDAL_SURFACE3D     | ENTITY_UCS 局部坐标系 double 值 r 为洞的中心到圆环面 ' 管 ' 的中心的距离 double 值 r 为圆环面 ' 管 ' 的半径          |
| B_SPLINE_SURFACE3D     | int u , v 次数 , 控制点数组 p 节点数组 u , v 节点重复度 w                                             |

## 1.3.3 数据转换

(1) 为每一类文件实例数据创建对应的实体 解析转换函数

文件中的实例数据都对应一种具体的数据结 构类型 , 因此转换函数实现的主要功能是针对每 一个文件数据 , 根据其所属的 STEP 实体类型与几 何引擎数据结构的映射关系 , 找到与其类型对应 的几何引擎数据结构 ; 然后对文件数据中包含的 属性值进行语义分析 ; 对现有的属性值进行分析 计算 , 得到几何引擎数据结构中所需的全部属性

表 4 STEP 标准几何类属性

| STEP 标准几何类                   | 包含属性                                               |
|------------------------------|----------------------------------------------------|
| CARTESIAN_POINT              | 3 个 double 值 x , y , z                             |
| DIRECTION                    | 3 个 double 值 x , y , z                             |
| VECTOR                       | CARTESIAN_POINT 三维点、 DIRECTION 方向、长度               |
| AXIS2_PLACEMENT_3D           | CARTESIAN_POINT 原点、 局部坐标系的 z 轴以及 x 轴               |
| LINE                         | CARTESIAN_POINT 三维点 VECTOR 方向矢量                    |
| CIRCLE                       | 局部坐标系、半径                                           |
| ELLIPSE                      | 局部坐标系、主轴长、短轴长                                      |
| B_SPLINE_CURVE_ WITH_KNOTS   | int 次数 p 控制点 节点数组 vector<double> 节点重复度 vector<int> |
| RATIONAL_B_SPLINE_ CURVE     | 权重 vector<double>                                  |
| PLANE                        | 局部坐标系                                              |
| CYLINDRICAL_ SURFACE         | CYLINDRICAL_SURFACE3D                              |
| CONICAL_SURFACE              | 局部坐标系 double 值 r 表示底面圆半径角度                         |
| SPHERICAL_SURFACE            | 局部坐标系 double 值 r 表示球半径                             |
| TOROIDAL_SURFACE             | 局部坐标系 洞的中心到圆环面 ' 管 ' 的中心的 距离 r 圆环面 ' 管 ' 的半径 r     |
| B_SPLINE_SURFACE_ WITH_KNOTS | int u , v 次数 控制点 节点数组 u , v 节点重复度 vector<int>      |
| RATIONAL_B_SPLINE_ SURFACE   | 权重 vector<vector<double>> w                        |

值 , 从而创建几何引擎中对应的实体类对象 . 拓扑 数据结构的转换函数如表 5 所示 , 几何数据结构的 转换函数如表 6 所示 .

表 5 拓扑数据结构转换函数

| 函数名                                  | 转换的数据结构类型           |
|--------------------------------------|---------------------|
| ParseManifoldSolidBrep(int p Index ) | ManifoldSolidBrep 类 |
| ParseShell(int p Index )             | Shell 类             |
| ParseClosedShell(int p Index )       | ClosedShell 类       |
| ParseFace(int p Index )              | Face 类              |
| ParseAdvancedFace(int p Index )      | AdvancedFace 类      |
| ParseFaceBound(int p Index )         | FaceBound 类         |
| ParseFaceOuterBound(int p Index )    | FaceOuterBound 类    |
| ParseLoop(int p Index )              | Loop 类              |
| ParseEdgeLoop(int p Index )          | EdgeLoop 类          |
| ParseOrientedEdge(int p Index )      | OrientedEdge 类      |
| ParseEdge(int p Index )              | Edge 类              |
| ParseEdgeCurve(int p Index )         | EdgeCurve 类         |
| ParseVertex(int p Index )            | Vertex 类            |
| ParseVertexPoint(int p Index )       | VertexPoint 类       |

表 6 几何数据结构转换函数

| 函数名                                   | 转换的数据结构类型              |
|---------------------------------------|------------------------|
| ParseSurface(int p Index )            | Surface3D 类            |
| ParseCylindricalSurface(int p Index ) | CylindricalSurface3D 类 |
| ParsePlane(int p Index )              | Plane3D 类              |
| ParseConicalSurface(int p Index )     | ConicalSurface3D 类     |
| ParseSphericalSurface(int p Index )   | SphericalSurface3D 类   |
| ParseToroidalSurface(int p Index )    | ToroidalSurface3D 类    |
| ParseRB_SplineSurface3d(int p Index ) | B_SplineSurface_3D 类   |
| ParseB_SplineCurve3d(int p Index )    | B_SplineCurve3D 类      |
| ParseCartesianPoint(int p Index )     | Point3D 类              |
| ParseCircle(int p Index )             | Circle3D 类             |
| ParseEllipse3d(int p Index )          | Ellipse3D 类            |
| ParseLine(int p Index )               | Line3D 类               |
| ParseAxis2Placement3d(int p Index )   | Entity_ucs 类           |
| ParseDirection(int p Index )          | Direction3D 类          |
| ParseVector(int p Index )             | Vector3D 类             |

## (2) 确定转换策略

文件数据之间具有嵌套的 ' 树形 ' 层次数据结 构关系 , 如图 4 所示 .

图 4    ' 树形 ' 层次图

<!-- image -->

按照这种层次结构关系特点 , 本文采取的转换 策略方法如下 . 针对每一个模型体 ,  ' 以模型体的 解析函数为入口 , 从下往上 ' 进行构建 . 具体做法 是先找到文件数据中表示模型体的实例数据 , 然

后执行相应的解析转换函数 . 由于模型体类中包 含壳的属性 , 因此创建模型体类对象前要先调用 其包含的壳实例数据的解析函数创建壳对象 . 同 理 , 壳中包含面 , 在创建壳对象前要先创建面对 象 , 依次往下 , 一直到层次结构的最底层 . 拓扑结 构转换函数的调用层次结构图如图 5 所示 .

图 5 拓扑结构转换函数的调用层次结构图

<!-- image -->

## 1.3.4 数据合法性检查

不同软件生成的 STEP 中性文件之间存在一 定的差异 , 同一个几何或拓扑数据的表达形式具 有多样性 . 除此之外 , 某些 CAD 软件在输出 STEP 中性文件时会采取一些操作使得读出的文件中的 数据信息不完整 . 因此 , 在对 STEP 中性文件的每 一行数据进行转换操作后 , 都需要对当前数据进 行拓扑完整性检查 , 保证模型拓扑的正确性以及 完整性 . 在数据合法的条件下 , 才能说明当前数据 已成功完成转换 .

## 1.4 当前框架存在的问题

通过前 3 小节描述的 STEP 中性文件读取算 法 , 可以成功地实现对 STEP 中性文件的读取 . 然 而 , 当读取的 STEP 中性文件过大时 , 就会出现文 件加载速度慢的问题 , 导致读取等待时间过长 . 本 文仔细分析读取流程以及中性文件中的数据特点 , 发现造成加载速度慢的原因主要有 2 点 .

## (1) 数据分析模块中对数据处理过慢

当前读取流程中数据分析模块对数据进行处 理时 , 采取的方法是逐行读取文件中的数据并处 理 . 这样读取文件中的数据存在一个弊端 , 即每一 次读取数据系统都需要从内存转到磁盘上去 . 由 于每次访问磁盘都需要花费很长的时间 , 因此多 次访问磁盘势必会浪费很多不必要的时间 .

## (2) 数据转换模块中数据实体转换速度过慢

由于拓扑几何数据之间具有层次依赖关系 , 因此在转换数据、 创建模型实体对象时采取的办法 是按照层次结构从上往下 ' 递归 ' 构建 , 一次只构 建一个实体对象 . 也就是说 STEP 中性文件中的实 例数据在转换成几何引擎内部的几何和拓扑实体 时是逐个转换的 , 一次只能转换一个实例 .

针对第 1 点引起的读取速度慢的问题 , 本文采 取的措施为改变文件读取方式 , 由原来的一次读 取一行数据转换成一次性读取整个文件数据 , 并 对读入数据预处理 .

具体处理做法如下 : 将文件中所有的数据一 次性读进内存 ; 由于数据在文件中可能占据多行 , 因此在对读入数据进行分割时不再按照常用的换 行符进行分割 , 而是根据数据的结束符来对数据 进行分割 ; 然后将结果存入数组中 , 每次从数组中 取一个数据进行处理 . 通过该方法可以有效地减少 CPU 与磁盘之间的访问次数 , 提高文件读取速率 .

针对第 2 点 , 当前读取算法每次转换时都只处 理一个实体对象 , 所有实体皆是按顺序一个接一 个地被解析并创建相应的实体对象 . 本文在分析 研究文件读取流程的基础上 , 结合文件数据特点 提出了基于多线程的分层并行处理算法 , 实现了 多数据同时转换 , 在提高读取转换速度的同时 , 提 高了整个程序执行效率 .

## 2 基于分层策略的并行处理算法

## 2.1 算法概述

分析 STEP 中性文件中的数据特点发现 , STEP 中性文件中的拓扑数据和几何数据之间具有 ' 树 形 ' 层次关系 , 位于低层的几何数据 ( 或拓扑数据 ) 会被位于它上层的拓扑结构数据引用 , 而同层的 拓扑数据或几何数据之间相互独立 , 互不依赖 . 基 于以上特点 , 本文提出了分层并行转换算法 , 将模 型中的数据进行分层存储 , 从底层数据开始 , 对处 于同一层次的数据进行并行解析 , 并统一存储解 析后的数据 . 按照这种方式逐层地向上构建出表 示完整模型的实体数据 . 这种方法需要借助额外 的数据结构才能实现并行处理功能 .

## 2.1.1 分层数据结构

本文将 ' 树形 ' 层次结构图中的数据按照它们 之间的层次依赖关系分成 10 层 , 每一层中包含的 几何或拓扑实体数据如表 7 所示 . 其中各层包含的 数据个数根据不同的 STEP 文件而变化 .

表 7 分层数据结构

|   层数 | 当前层包含的几何或拓扑数              |
|------|---------------------------|
|    1 | Point, Direction          |
|    2 | Vertex, Placement, Vector |
|    3 | Curve, Surface            |
|    4 | Edge                      |
|    5 | OrientedEdge              |
|    6 | Loop                      |
|    7 | FaceBound                 |
|    8 | Face                      |
|    9 | Shell                     |
|   10 | ManifoldSolidBrep         |

第 1 层由底层中的数据组成 , 包括几何实体实 例数据笛卡儿点和方向 , 这 2 种几何实体数据是其 他所有数据建立的基础 , 所以需要首先构建这 2 种 数据 . 接下来是 Vertex, Placement 和 Vector 等实体 数据的构建 , 这 3 类实体只包含 Point 或 Direction 实体 , 而这 2 种类型数据均已在第 1 层构建完成 , 可直接调用 . 然后构建几何实体数据 Curve 和 Surface 类型 , 这 2 种实体所包含的实体数据同样 在前 2 层构建完成 . 同理 , 可将其他层上的拓扑几 何数据进行构建 , 最后一层构建的实体便是代表 一个模型的实体数据 .

## 2.1.2 线程互斥情况分析

在这种分层数据结构中 , 实体数据的构建都 是从第 1 层开始、 从下到上分层进行 . 在该结构中 , 不用具体区分哪个模型包含哪些数据 , 每一层数 据之间的构建都是相互独立的 , 只要保证在创建

当前层的数据时 , 前几层的数据已完成创建即可 . 2.1.3 具体算法实现

输入 . STEP 中性文件 .

输出 . 模型体对象 .

Step1. 一次性加载 STEP 中性文件 , 对数据进行分 割后存储 .

Step2. 对存储的数据进行分析 , 根据数据所属的 实体类型对数据按层存储 .

Step3. 创建线程池 , 按层遍历 , 对该层中的数据并 行创建 .

Step4. 对每一个数据的合法性进行检查 .

Step5. 将创建完成的数据的地址存放到一个一维 数组中 , 存放的位置与数据的索引 ( 在 STEP 中性文件中 的序号 ) 保持一致 .

Step6. 判断是否所有层数据均已创建 , 若未完成 , 转 Step4; 否则执行下一步 .

Step7. 返回最后一层中的数据 , 即输出创建好的 所有模型体对象的地址 .

## 2.2 并行算法实验

本实验是在 Windows 环境下 , 利用 Visual Studio  2019 集成开发环境完成的 . 实验使用的电 脑 CPU 为 Intel(R)  Core™  i5-8265U,  8 核 . 实验结 果中的时间因使用的主机配置不同可能会有些差 异 , 但实验结论不变 .

本文对大量文件样例进行了实验 , 按照文件 大小分成了 4 个区间 :  5~15 MB,  45~55 MB,  95~ 105 MB,  145~155 MB. 表 8 给出了每个区间中比 较具有代表性的文件样例信息 , 包括每一个文件 包含的点、面和边的个数 . 本文展示了部分文件的 模型图 , 如图 6 所示 .

表 8    STEP 中性文件样例

| 文件名            | 简称     |   文件大小 /MB | 点数 / 个   | 边数 / 条   | 面数 / 个   |   模型数 / 个 |
|----------------|--------|------------|----------|----------|----------|-----------|
| REVA 汽车转换器组装设备 | REVA   |        148 | 694 617  | 114 902  | 43 692   |       823 |
| 智能机器人          | 机器人    |        108 | 466 756  | 89 037   | 39 371   |       387 |
| Q12 直升机        | 12 直升机 |         56 | 412 085  | 12 174   | 4 591    |        35 |
| Q11 直升机        | 11 直升机 |         12 | 39 560   | 10 732   | 3 921    |        47 |

<!-- image -->

a. REVA 汽车转换器组装设备

b. Q11 直升机

<!-- image -->

c. 智能机器人

<!-- image -->

<!-- image -->

d. Q12 直升机

图 6    STEP 模型

首先 , 在进行并行处理实验初始阶段 , 本文先 验证了 CPU 的核数和线程数对实验算法的影响 , 从而在最优的条件下进行对比实验 .

针对 CPU 核数对并行算法的影响 , 本文分别 测试了在 2 核、 4 核、 6 核以及 8 核状态下对并行 处理算法读取效率的影响 . 图 7 给出了在不同核数 下并行算法读取 STEP 中性文件的读取时间变化过 程 . 通过图中走向可以看出 , 在 CPU 核数较少时 速率提升较快 , 核数增多时读取速度逐渐收敛 .

图 7    CPU 核数对 STEP 文件读取效率的影响

<!-- image -->

针对线程数对并行算法的影响 , 本文选取了 不同数量的线程对文件大小在 5~15MB 的模型进 行测试 , 实验结果如图 8 所示 . 可以看到线程数从 1~6 对读取速度有较明显的提升 , 随着线程的不断 增加 , 对读取效率的影响逐渐降低 .

图 8 线程数对 STEP 文件读取效率的影响

<!-- image -->

图 9    STEP 中性文件读取效率对比

<!-- image -->

本文采用并行处理方法对文件读取时间进行 测试 , 同时与 OpenCASCADE 的读取时间进行了 对比 . 图 9 所示为并行处理前后的代码 ( 分别简写 为并行前和并行后 ) 以及 OpenCASCADE 读取 STEP 文件的时间对比 .

模型文件越大 , 文件中的模型包含的零件模 型往往越多或模型越复杂 . 本文选取多组大小相 似的文件 , 对模型文件中包含的模型个数 , 面的数 量对分层并行算法的效率产生的影响进行实验 . 表 9 所示为其中一组模型文件的读取时间结果 .

表 9    STEP 中性文件读取结果

| 文件名   |   文件大小 /MB |   模型数 / 个 | 面数 / 个   |   运行时间 /s |   OpenCASCADE 读取时间 /s |
|-------|------------|-----------|----------|-----------|-----------------------|
| 侦察车   |        148 |       817 | 73 692   |   497.365 |                 743.4 |
| 热熔机   |        143 |       564 | 90 489   |   502.731 |                 763.2 |
| 机械手   |        150 |       792 | 70 467   |   496.694 |                 749.6 |
| 平衡机   |        152 |       513 | 87 352   |   504.697 |                 759.5 |

## 2.3 实验结论

对表 8 、表 9 以及图 9 中数据进行分析 , 可得 以下结论 .

- (1) 应用并行处理算法后 , 文件的读取速度明 显提升 , 相比并行前读取文件的运行速率提升至 2 倍 . 与 OpenCASCADE 读取算法相比 , 比其快 50%.
- (2) 对于分层并行算法 , 模型数和面数对算法 的影响较小 . 分层算法关注更多的是各个层次包

含的数据结构对象的数量 , 与模型数和面数并没 有直接的关系 .

## 3 结 语

本文提出了一种基于多线程并行加速的 STEP 中性文件读取算法 . 针对当前算法中存在的文件 读取速度过慢的问题 , 提出了基于数据分层的并

行加速策略 . 不同层数据之间的构建相互独立 , 同 一层之间的数据利用线程池进行并行加速 . 经实验 证明 , 优化算法在一定程度上提高了文件读取速度 .

由于不同软件对 STEP 文件的处理方式不同 , 针对不同的软件导出的 STEP 文件则需要做特别的 处理 . 本文算法仅支持 SolidWorks 以及 Catia 软件 读出的 STEP 文件 , 后续将会针对其他 CAD 软件 读出的文件做进一步的处理 , 以便于增强算法的 正确性和鲁棒性 .

## 参考文献 (References):

- [1] Yue Longwang, Hua Haifeng, Wang Xiaoyan, et al . Analysis of STEP  standard  hierarchy[J].  Machinery  Industry  Standardization &amp; Quality, 2013(1): 23-26(in Chinese) ( 岳龙旺 , 华海峰 , 王晓燕 , 等 .  STEP 标准层次结构分析 [J].
2. 机械工业标准化与质量 , 2013(1): 23-26)
- [2] Jiang Meng, Lü Huiqiang. Research on standard for exchange of  product  model  data[J].  Computer  Engineering  and  Design, 2008, 29(13): 3542-3545(in Chinese) ( 姜萌 , 吕慧强 . 产品模型数据交换标准 STEP 研究 [J]. 计算
4. 机工程与设计 , 2008, 29(13): 3542-3545)
- [3] Chen  Deren.  Product  data  representation  and  exchange  standard STEP and its integration technology[J]. Science &amp; Technology Review, 1997(1): 30-33(in Chinese) ( 陈德人 . 产品数据表达与交换标准 STEP 及其集成技术 [J].
6. 电子科技导报 , 1997(1): 30-33)
- [4]  Wang Guoxun, Wang Jun, Sun Jun. Mapping from EXPRESS to  C++  Schema[J].  Mechanical  Engineer,  2004(7):  22-24(in Chinese) ( 王国勋 , 王军 , 孙军 .  EXPRESS 描述到 C++ 模式映射的研 究 [J]. 机械工程师 , 2004(7): 22-24)
- [5]  Dong Weiguang. CAD product data exchange based on STEP standard[D].  Lanzhou:  Lanzhou  University  of  Technology, 2005(in Chinese) ( 董唯光 . 基于 STEP 标准的 CAD 产品数据交换 [D]. 兰州 :
9. 兰州理工大学 , 2005)
- [6] Ma Zhiyuan. Research and application of product data sharing and exchange technology based on STEP[D]. Wuhan: Wuhan University of Technology, 2004(in Chinese) ( 马志远 . 基于 STEP 的产品数据共享与交换技术的研究与 应用 [D]. 武汉 : 武汉理工大学 , 2004)
- [7] Tao  Yuping.  Research  on  correlation  between  STEP  standard and  GPS  standard[D].  Dalian:  Dalian  Jiaotong  University, 2005(in Chinese)
12. ( 陶玉萍 . STEP 标准与 GPS 标准相关性研究 [D]. 大连 : 大连 交通大学 , 2005)
- [8] Wu Huibo.  Study  on  creating  STEP  file[J].  Journal  of  Hebei University of Technology, 2013, 42(4): 69-72(in Chinese)
14. ( 吴会波 .  STEP 文件创建研究 [J]. 河北工业大学学报 ,  2013, 42(4): 69-72)
- [9] Zhang  Sirong,  Tan  Jianrong,  Zhang  Jiatai, et  al .  The  implementation  methods  of  STEP  neutral  file[J].  Journal  of  Computer Aided Design &amp;  Computer  Graphics, 1999, 11(1): 43-48(in Chinese)
16. ( 张思荣 , 谭建荣 , 张家泰 , 等 . STEP 中性交换文件的实现方 法 [J]. 计算机辅助设计与图形学学报 , 1999, 11(1): 43-48)
- [10] Qiu  Changhua,  Xue  Kai,  Zhu  Shifan, et  al .  Implementation method  of  STEP  file[J].  Applied  Science  and  Technology, 1996(1): 16-19(in Chinese) ( 邱长华 , 薛开 , 朱世范 , 等 .  STEP 文件的实施方法 [J]. 应用
18. 科技 , 1996(1): 16-19)
- [11] Zhu Dapei, Xu Yong'an, Yang Qin, et al . Data exchange based step  neutral  file[J].  Computer  Engineering  and  Design,  2001, 22(4): 5-8(in Chinese) ( 朱大培 , 徐永安 , 杨钦 , 等 . 基于 STEP 标准的数据交换的
20. 研究与实现 [J]. 计算机工程与设计 , 2001(4): 5-8)
- [12] Zhao Fuqing, Yu Dongmei, Yang Yahong. The study and application of implementation method for STEP[J]. Journal of Engineering Graphics, 2002, 23(4): 1-8(in Chinese) ( 赵付青 , 余冬梅 , 杨亚红 .  STEP 标准实现方法的研究与应
22. 用 [J]. 工程图学学报 , 2002, 23(4): 1-8)
- [13] Huang Zhongdong, Yang Xiaohu, Dong Jinxiang. Research on key  techniques  of  SDAI  implementation[J].  Journal  of  Computer  Aided  Design  &amp;  Computer  Graphics,  2001,  13(11): 977-982(in Chinese)
24. ( 黄忠东 , 杨小虎 , 董金祥 .  SDAI 实现中若干关键技术的研 究 [J]. 计算机辅助设计与图形学学报 , 2001, 13(11): 977-982)
- [14]  Duan Guanghong, Liu Dan, Wang Junying, et al . Research and realization  of  object-oriented  CAD/CAPP/CAM  integration system based on STEP[J]. Computer Integrated Manufacturing Systems, 1997(6): 40-44(in Chinese)
26. ( 段广洪 , 刘丹 , 王君英 , 等 . 基于 STEP 标准的面向对象的 CAD/CAPP/CAM 集成系统的研究与实现 [J]. 计算机集成制 造系统 , 1997(6): 40-44)
- [15] Yang Juan, Cui Fengkui. Integration technology of CAD/CAPP based on STEP - Standard in PDM platform[J]. Machine Tool &amp; Hydraulics, 2004(4): 31-32(in Chinese) ( 杨娟 , 崔凤奎 .  PDM 平台下基于 STEP 标准的 CAD/CAPP
28. 集成技术 [J]. 机床与液压 , 2004(4): 31-32)
- [16]  Yue Longwang, Liu Baoguo, Wang Xiaoyan, et  al .  CAX system  integration  technology  research  based  on  the  STEP  and VC++[J].  Manufacturing  Automation,  2013,  35(8):  91-95(in Chinese)
30. ( 岳龙旺 , 刘保国 , 王晓燕 , 等 . 基于 STEP 和 VC++ 的 CAX 系统集成技术研究 [J]. 制造业自动化 , 2013, 35(8): 91-95)
- [17] Li  Peilin.  Research  of  CAD/CAE  integration  based  on  STEP standard[D].  Harbin:  Harbin  Engineering  University,  2009(in Chinese)
32. ( 李佩霖 . 基于 STEP 的 CAD/CAE 系统集成技术的研究 [D]. 哈尔滨 : 哈尔滨工程大学 , 2009)