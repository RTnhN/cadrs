## 옵셋팅을 위한 정규 삼각망 추출

, * 정원형 정춘석 , * 신하용 , * 최병규 *

## Extracting a Regular Triangular Net for Offsetting

Jung, W.H. , * Jeong, C.S. , * Shin, H.Y. * and Choi, B.K. *

## ABSTRACT

In this paper, we present a method of extracting a regular 2-manifold triangular net from a triangular net including degenerate and self-intersected triangles. This method can be applied to obtaining an off­ set model without degenerate and self-intersected triangles. Then this offset model can be used to gener­ ate CL curves and extract machining features for CAPR The robust and efficient algorithm to detect valid triangles by growing regions from an initial valid triangle is presented. The main advantage of the algorithm is that detection of valid triangles is performed only in valid regions and their adjacent self­ intersections, and omitted in the rest regions (invalid regions). This advantage increases robustness of the algorithm. As well as a k-d tree bucketing method is used to detect self-intersections efficiently.

Key words : Regular triangular net, Regular triangular net extraction, Offset, Self-intersection, k-d tree

## 1 . 서 론

본 연구에서는 offset 등에 의해서 기하학적으로 변 형된 삼각망으로부터 퇴화 삼각형과 self-intersection 에 의해 형성된 무효 영역이 제거된 2-mainfold 경계 를 구하는 정규 삼각망 추출 (regular triangular net extraction) 방법을 제안한다. 이 논문에서는 정규 삼각 망을 퇴화 삼각형과 self-inter-section°] 존재하지 않는 2-manifold 경계의 삼각망으로 정의한다. 또한 단일 부 피를 정의하는 2-manifold 경계가 기하학적으로 변형 된 삼각망에 대한 정규 삼각망 추출로 연구 범위를 한 정한다.

정규 삼각망 주줄 (regular triangular net extraction) 방법은 삼각망 모델 기반의 CAM 시스템에서 CL (cutter location) 곡선을 생성하거나 CAPP 를 위한 machining feature 정의 (Choi et 를 위해서 삼각 망 마스터 모델로부터 CL 곡면 모델인 o&amp;set 모델을 계산하는데 적용할 수 있다.

퇴화 삼각형과 self-intersection 을 제거하는 정규 삼 각망 추출은 intersection 을 계산하고 그것으로부터 필 요한 영역을 주출해낸다는 점에서 Boolean operation

*KAIST 산업공학과

- 논문투고일:

- 심사완료일:

(Cardan and Perrin 「기)과 유사하다. 하지만 Boolean operation 은 2 개의 solid 개체를 대상으로 하기 때문에 intersection 의 계산과 필요한 영역의 추출은 본 연구 의 정규 삼각망 추출보다 용이 하게 수행될 수 있다.

의류 시뮬레이션과 같은 변형 가능 물체 (deformable object) 의 시뮬레이션。쇼은 self-collision 탐색과 collision response 문제를 다룬다. 이것은 self-collision 을 효과 적으로 탐색해야 한다는 점에서 본 연구의 문제와 유 사하다. 하지만 self-collision 을 발생시키는 부분을 제 거하는 것이 아니고 그 위치를 재정의 해주는 문제를 다루며, 시뮬레이션을 위한 다중 프레임 계산 과정에 서 인접 프레임간에 self-collision 계산은 프레임들 간 의 유사성을 이용하여 계산의 효율성을 높일 수 있다 는 점에서 차이가 있다.

정규 삼각망 추출은 먼저 무효 삼각형을 탐색해서 제거하는 방식을 생각할 수 있다. Offset 등의 변형에 의해 self-intersection 이 발생하는 영역에서는 삼각형 들이 매우 복잡하게 얽혀있는 부분이 많고 이러한 영 역 내부의 self-intersection segment 들은 유효 삼각형 과 제거되어야 할 무효 삼각형의 경계 역할을 하지 않 고 무효 삼각형에 의해서만 형성된 것들이 대부분이 기 때문에 무효 삼각형의 탐색과 판단은 쉽지 않다. 따라서 본 연구에서는 먼저 유효 삼각형들을 탐색한 후에 탐색되지 않은 삼각형들을 제거하는 방식을 사

<!-- image -->

용하目 무효 영역 내에서는 불필요한 유효, 무효 삼각 형 판단을 수행하지 않으면서 단순화된 유효, 무효 삼 각형의 판단이 가능한 방법을 제안하고자 한다.

## 2. 전체 수행 절차

Fig. 1 에서와 같이 다음의 5 단계를 거쳐서 정규 삼 각망을 추출한다.

## Step 1. 퇴화 삼각형 제거

퇴화 삼각형은 면적이 zero tolerance 이내인 삼각

Fig. 1. An explanatory example of overall procedure.

<!-- image -->

Fig. 2. Deletion of degenerate triangles.

<!-- image -->

형으로 삼각망의 변형으로 인해 발생할 수 있다. 퇴화 삼각형의 제거는 이후의 각 단계의 알고리즘 수행을 위해 우선적으로 수행되어야 한다. 퇴화 삼각형 知.는 최소 길이 edge I &lt; e^Zero length tolerance) 인 삼각 형과 최소각 a &lt; £2(Zero angle tolerance) 인 삼각형으 로 정의한다. 최소 길이 edge /&lt;&amp;인 삼각형은 Fig. 2(a) 와 같이 edge collapse 에 의해 제거되고, 최소각 a &lt; £2(Zero angle tolerance) 인 삼각형은 Fig. 2(b) 와 같이 최대 길이의 edge 를 swapping 하여 제거한다. 삼 각망의 모든 삼각형에 대해서 퇴화 삼각형 테스트를 수행한다. 각 퇴화 삼각형의 제거 후에는 주변 삼각형 들의 위상 (topology) 정보가 수정되어야 한다.

## Step 2. Self-intersection 계산

두 삼각형간의 intersection 테 스트를 통해 서 intersection segments 들을 계산하고 리스트에 저장한 다이때 intersection segment 는 양 끝점에 대한 위치 (position) 와 intersection 을 만든 두 삼각형 정보를 저 장한다. 상세한 계산 방법은 3 장에서 설명한다.

## Step 3. 초기 유효 삼각형 탐색

정규 삼각망 추출을 통해서 남게 될 유효 영역의 삼각형 중에 하나를 탐색하는 과정이다. 유효 영역이 항상 최외곽에 존재한다는 것이 보장되는 경우에는

1.  X, y, z 의 최대 또는 최소 값을 가지는 vertex 중 임의의 한 vertex v 를 선택

2.  V 에 연결된 삼각형 중에서 임의의 삼각형 t 선택 3.  f 가 다른 삼각형과 intersection 이 없는 경우에는 t 를 초기 유효 삼각형으로 선택하고, 다른 삼각 형과 intersection 이 있는 경우에는 r 를 subtriangulation 한 후 v 에 연결된 sub-triangle 중 임 의의 한 삼각형 项 ,를 유효 삼각형으로 선택 한다. 유효 영역이 최외곽에 존재하지 않는 경우에는 사용자의 선택에 의해서 초기 유효 삼각형을 결정한다.

## Step 4. 유효 영역 탐색

하나의 유효 삼각형에 인접한 self-intersection 이 없 는 삼각형도 역시 유효 삼각형이다. 유효 영역의 탐색 은 Fig. 1 에서 보는 것처럼 Step 3 에서 선택된 초기 유효 심긱■형에서부터 self-inter-section segment 를 가 진 삼각형을 만날 때까지 인접 삼각형으로 유효 삼각 형들의 영역을 확장해 나가는 방식이다.

단일 부피를 정의하는 2-manifold 경겨)가 기하학적 으로 변형된 삼각망의 경우 하나의 유효 영역은 항상 self-intersection segment 들로 그 경계가 한정된다. 따

라서 Fig. 1 의 유효 영역 탐색 과정에서 보는 것처럼 유효 영역을 한정하는 self-intersection segment 를 통 해서 인접한 다른 유효 영역으로 넘어갈 수 있다. 이 러한 과정을 반복해서 유효 영역을 한정하는 모든 self-intersection segment 에서 유효 삼각형이 탐색되면 유효 영역 탐색이 완료된다.

이러한 방식의 유효 영역 탐색은 무효 영역에 속하 는 self-intersection segment 들에서는 유효/무효 삼각 형 판단 및 탐색을 전혀 수행하지 않는다. 유효 영역 탐색에 대해서는 4 장에서 자세하게 설명한다.

## Step 5. Trimming &amp; Stitching

Step 4 에서 얻어진 유효 영역의 경계를 한정하 는 self-in 杞 ^section 들에서 sub-triangulation 을 통 한 trimming 과 self-intersection segment 를 기준으로 segment°l] 인접한 sub-triangle 들의 인접 관계를 설정 해주는 stitching 을 수행하고 탐색되지 않은 무효 삼각 형과 무효 sub-tri-angle 들을 제거하는 과정을 수행하 면 된다.

Step 4 의 유효 영역 탐색 과정에서는 4.2 절과 43 절 에서 설명할 것처럼 유효 영역의 경계를 한정하는 self-intersection segment 포함하는 삼각형들에서 sub-triangulatior 과 이를 통해 생성된 sub-triangle 들에 대한 유효 삼각형 탐색이 수행되기 때문에, 이 단계에 서는 단순히 이러한 self-intersection segment 를 기준 으로 seg-ment 에 인접한 sub-triangle 들의 인접 관계를 설정해 주고 탐색되지 않은 무효 삼각형과 무효 subtriangle 들을 제거하는 과정 만을 수행함으로써 정규화 된 삼각망을 얻을 수가 있다.

## 3.  S 이 intersection 계산

삼각망 모델의 self-intersection 을 계산하기 위해서 는 삼각형들간에 intersection 테스트가 필요한데 삼각 형들의 모든 쌍에 대해서 intersection^ 테스트한다면 O(M) 의 complexity 가 요구된다. 따라서 지역화 (localization) 하여 테스트할 삼각형들의 쌍을 제한할 필요가 있다. 이 와 관련하여 삼각망 모델을 Self-inter­ section0] 존재하지 않는 영역으로 clustering 하여 계산 의 효율성을 높인 연구 (\blino et 瓦」히)가 있다. 본 연 구에서는 k-d treH 기를 사용하여 삼각형들을 bucketing 한 후에 k-d tree 의 leaf node 내에 있는 삼각형들간에 만 intersectiori 테스트를 수행하는 방법을 사용하였 다. 아래의 단계를 거쳐 삼각망 모델을 k-d tree 로 bucketing 한다 (Fig. 3).

Fig. 3・ k-d tree bucketing.

<!-- image -->

A

1.  x, y, z 축에 평행하도록 삼각망 모델의 bounding box 를 계산하고, 모든 삼각형을 bounding box 의 삼각형 리스트에 저장한다.
2. bounding box 의 각 모서리의 길이를 비교, 가장 긴 방향으로 bounding box 를 2 등분하여 2 개의 하위 bounding box 를 생성한다.
3. 상위 bounding box 의 삼각형 리스트에 저장된 모든 삼각형에 대해서 각 하위 bounding box 와 intersection 이 발생하는 삼각형을 해당 bounding box 의 삼각형 리스트에 삽입한다.
4. 하위 bounding box 를 자신에 속한 삼각형들을 포함하는 최소 크기로 조정한다.
5. 자신에 속한 삼각형의 개수가 bucket capacity 이 상인 하위 bounding box 에 대해 2,3,4,5 번 과정 을 반복한다.

삼각망 모델을 k-d tree 로 buckeling 한 후에는 k-d tree 의 leaf node 별로 그것에 속한 모든 삼각형간에 inter-section 테스트 (M 이 1 迎幻)를 수행한다.

Fig. 4 는 k-d tree 를 사용하는 경우의 계산 시간을 그래프로 표현한 것인데 거의 선형적으로 증가한다는 것을 보여주고 있다. 하지만 Fig. 5 에서 볼 수 있는 것처럼 k-d tree 의 bucket capacity 에 따라서 동일 모 델에 대한 self-intersection 계산 시간이 달라진다. 최

Fig. 4. Analysis of computation time (model size).

<!-- image -->

Fig. 5. Analysis of computation time (bucket capacity).

<!-- image -->

적의 bucket capacity 는 모델의 크기 및 형상에 따라 다양하고 최적값의 결정은 어려운 일이다. 또한 삼각 형의 크기가 불균일한 경우 크기가 큰 삼각형은 여러 개의 leaf node 에 포함되기 때문에 균일한 크기의 모 델보다 계산 시간이 증가하게 된다.

## 4. 유효 영역 탐색

## 4.1 유효 영역 탐색 방법

단일 부피를 정의하는 2-manifold 경계가 기하학적 으로 변형된 삼각망의 경우 하나의 유효 영역은 항상 self-intersection segment 들로 그 경계가 한정된다. 따 라서 모든 self-intersection segment 가 얻어진 상태에 서 초기 유효 삼각형에서부터 self-intersection 을 가진 삼각형을 만날 때까지 인접 삼각형으로 유효 삼각형

(a) Input raw offset T-mesh.

<!-- image -->

들의 영역을 확장해 나가면 self-intersection segment 에 의해서 한정되는 하나의 유효 영역을 탐색할 수 있 다. 이때 유효 영역의 경계를 이루는 self-intersection segment 는 또 다른 유효 영역과 접해있는 부분이 될 수 있다. 따라서 이러한 self-intersection segment 를 통해서 인접한 다른 유효 영역으로 넘어갈 수 있다. 이러한 과정을 반복해서 유효 영역을 한정하는 모든 self-intersection segment 에서 유효 삼각형이 탐색되면 유효 영역 탐색이 완료된다. 이 과정에서 탐색되지 않 은 삼각형들은 모두 무효 영역에 속하는 삼각형들이 된다.

이 과정은 Fig. 6 에서와 같이 단순화된 형태로 설명 될 수 있다. (a) 는 offset 에 의해 변형된 삼각망 모델 이고 (f) 는 추출된 정규 삼각망이다. 가운데 부분에서 self-intersection 에 의해 생긴 무효 영역이 제거되었음 을 알 수 있다. (b)~(e) 는 유효 영역의 탐색과정을 (a) 에 대한 2D 단면 형태로 단순화하여 설명한 그림이 다. (b) 에서 단면 경 계면에 수직 한 화살표들은 삼각형 의 법선 방향, 회색점은 self-intersection segment# 나타내며, 경계면 상의 검은색의 굵은 화살표는 초기 유효 삼각형을 나타내고 있다. (c) 는 (b) 의 유효 삼각 형에서부터 intersection 을 가진 삼각형을 만날 때까지 인접 삼각형으로 유효 삼각형들의 영역을 확장해 가 서 self-intersection segment 에 의해서 경계가 한정되 는 하나의 유효 영역을 탐색한 결과이다. (d) 는 (c) 에 서 탐색된 유효 영역의 경계를 이루는 한 self­ intersection segment 에서부터 인접한 다른 유효 영역

(b) Start from the seed triangle.

<!-- image -->

응 ion grows across the intersection.

<!-- image -->

<!-- image -->

(c) Valid region grows to intersections, (d) Valid re

<!-- image -->

(e) Complete valid region growing.

(f) Complete v 시 id region growing.

<!-- image -->

Fig. 6. Procedures of exploring valid regions.

<!-- image -->

o

Fig. 7. Sub-triangulation.

을 탐색한 결과이다. 이 과정은 4.2 절과 4.3 절에서 자 세히 설명된다. (d) 의 과정이 반복되어 모든 유효 영 역의 삼각형이 탐색되면 (e) 에서와 같이 무효 영역을 제거된 정규 삼각망 모델을 얻게된다.

## 4.2 Sub-triangulation

유효 영역 탐색 과정에서 유효 영역의 경계를 이루 는 self-intersection segment# 포함하는 삼각형들은 Fig. 7 과 같이 삼각형의 세 edge 와 intersection segment 들을 가지고 sub-triangulatioir 을 수행한다. 본 연구에서는 2D constrained Delaunay triangulation (R 叩 pe * 이) 방법을 사용 하였다.

이러한 sub-triangulation 결과 탐색된 유효 삼각 형에 접해있는 sub-triangle 은 역시 유효 삼각형이 되며 4 」에서의 유효 영역 탐색과 마찬가지로 이 삼 각형에서부터 intersection segment 를 만날때까지 인 접한 sub-triangle 로 유효 영역을 확장함으로써 subtriang 나 lation 영역에서 유효 삼각형을 탐색한다.

Intersection segment 를 가지는 입력 모델의 삼각형 인 base triari 읺 e 은 각각 sub-triangulation 과정을 통 해서 자신의 sub-triangular net 을 가지게 되는데 이 과정은 다음과 같다 (Fig. 7).

1. Base triangle 7 ； , 의 intersection segment {이，= 1, 끼에 대해서 다음을 수행한다.
- (1) intersection segment sh 呂와의 교점을 계산, 분할, 财 ，品 2, 饥 ^(intersection point) 생성
3. (2)% 呂를 공유하는 {4} 에 %, 昂 2, 吕 1, 呂 2, 内 를 할당
2. 政의 edge {e,\i = 1, …, 3} 에 대해서 다음을 수 행한다.
5. ⑴ e 상의 intersection point {pj\j = 1, m} 들 로。를 분할, {exk\k 드 1, m+1} 생성 (2)e 의 인접 삼각형에 {强}를 할당
3.  7 ， 의 sub-triangular net 생성한다.
- (1) 입력: base triangle vertex (v,), non split base triangle edge {$}, split edge {esk}, intersection segment (5W}, intersection point {Pn]
- (2) 2D constrained Delaunay triangulation
4. Non split base triangle edge {q}, split edge {&amp;}, inter-section segment {&amp;J 와 sub-triangular net 의 edge {。戒}와의 대응 관계를 설정 한다.

4 에서 생성된 정보는 sub-triangulation 영역에서 유 효 삼각형 탐색과 함께 유효 영역 탐색이 완료된 후에 trimming &amp; stitching 단계에서 사용하게 된다.

## 4.3  S 이「 intersection segment 에서 인접 유효 영 역의 탐색

4.2 절에서 설명한 것처럼 하나의 유효 영역이 탐색 될 때 intersection segment 를 포함하는 삼각형들은 sub-triangulation 된 후 이 영역에서도 sub-triangle 에 대한 유효 영역 탐색이 수행되어 self-inter-section segment 들이 유효 영역의 경계가 된다. 이러한 self­ intersection segment 에서 인접한 다른 유효 영역을 탐 색하게 된다.

Fig. 8 의 (a) 는 하나의 유효 영역 탐색이 완료된 상 태이다. self-intersection segment s 는，二과 7 ； 에 의해 생성되었다. (b) 〜 (d) 는 이 영역에 인접한 다른 유효 영 역을 탐색하는 과정을 설명하고 있다. y 를 포함하는 다른 삼각형 4 가 sub-triang 니 ation 되면 s 에는 7 ； 의 sub-triangle 儿 时 와 7 ； 의 sub-triangle h, 払 가 접하게 된다.

은 유효 삼각형으로 탐색된 상태이기 때문에 s 에 접한 Z 의 sub-triangle Z3, 以중 하나가 다른 유효 영 역의 삼각형 후보가 된다. 이때 삼각망 모델의 외부를 나 타내는 sub-triangle 의 법선 방향이 일정하게 유지 되도록 하기 위해서 (b) 에서와 같이 以를 유효 삼각형 으로 선택한다. 그러면 다시 払 에서부터 인접 subtrian 이 e 들로 유효 영역을 확장해 나가면서 또 하나의 유효 영역을 탐색하게 된다. 이 과정은 4.1, 4.2 절의

Fig. &amp; Exploring an adjacent valid region.

<!-- image -->

유효 영역 탐색 과정이 동일하게 적용된다. (c)~(d) 는 爲에서 ?5, T 、로 유효 삼각형을 탐색해 나가는 과정을 보여주고 있다.

## 5. 적용 결과

Fig. 9 는 유효 영역을 탐색한 후에 탐색되지 않은

Fig. 9. An example of regular triangular net extraction,

<!-- image -->

(c) Invalid 珀요 ion left after the vdid region growing (d) Regular T-mesh

<!-- image -->

Fig, 10. Bunny.

Fig. 13. Pump.

<!-- image -->

무효 삼각형들을 제거하는 Trimming &amp; stitching 예 를 보여주고 있다.

Fig. 10 〜 13 은 초기 삼각망 모델과 이를 row offset

한 모델에서 self-intersection 를 계산한 후 유효 영역 을 탐색한 후에 남겨진 무효 영역과 무효영역을 제거 한 후에 Trimming &amp; stitching 을 하여 얻어진 정규

Table 1. The experimental results

|             |                |                            | Time (sec)                    | Time (sec)                 | Time (sec)   |
|-------------|----------------|----------------------------|-------------------------------|----------------------------|--------------|
| Name        | # of Triangles | # of Intersection Segments | Self-intersection Calculation | Region growing & Stitching | Total        |
| Bunny       | 15,224         | 4,253                      | 2                             | 0.454                      | 2.454        |
| Knot        | 150,776        | 44,211                     | 11.766                        | 0.656                      | 12.422       |
| C-arm lower | 173,880        | 14,141                     | 19.266                        | 2.591                      | 21.857       |
| Pump        | 256,672        | 35,401                     | 50.781                        | 3.140                      | 53.921       |

삼각망을 생성한 예들을 보여준다.

Table 1 에서의 적용 예제에 대한 수행 시간을 보여 주고 있다. Table 1 에서 볼 수 있는 것처럼 self­ intersection 계산 시간이 수행 시간의 대부분을 차지 함을 알 수 있다. 펜티엄 4 1.8GB, 1GB 의 'PC 에서 테 스트하였다.

## 6. 결론 및 추후 과제

단일 부피를 정의하는 2-manifold 경계가 기하학적 으로 변형된 삼각망으로부터 퇴화 삼각형과 selfintersection 에 의해 형성된 무효 영역이 제거된 2manifold 의 삼각망을 추출하기 위해서 하나의 유효 삼각형으로부터 영역을 확장해나가는 방법을 제안하 였다. 이 방법은 무효 영역 내에서는 불필요한 유효, 무효 삼각형 판단을 수행하지 않으면서 단순화된 유 효, 무효 삼각형의 판단이 가능하기 때문에 효율성과 함께 robustness 를 높이고 구현이 용이하다는 장점이 있다.

모델에 따라서 하나의 초기 유효 삼각형으로 모든 유효 영역을 탐색하지 못할 수 있다. 초기 유효 삼각 형 탐색 방법과 함께 이러한 점을 고려한 보다 일반 화된 유효 영역 탐색 방법의 연구가 필요하다. 이와 함께 점, 면이 접하는 경우등의 특이 상황을 고려한 연구가 실제 응용 측면에서 중요하게 요구되기 때문 에 이에 대해서 다양한 테스트를 통한 연구를 추가로 진행할 계획이다. 또한 5 장의 적용 결과에서 볼 수 있는 것처럼 수행 시간의 대부분이 self-intersection 계산에 소요된다. 따라서 대용량 모델에 적용하기 위 해서는 무효 영역으로 판단되는 부분에서는 가능한 한 self-intersection 계산 자체를 생략할 수 있는 방 법의 개발이 필요하다. 그리고 이와 함께 CAD 모델 에 대한 STL 파일처럼 삼각형의 크기의 불균일 정도 가 큰 경우에는 계산 시간이 크게 증가하는데 이를 함께 고려한 self-intersection 계산 방법을 연구할 계 획이 다.

## 감사의 글

본 연구는 과학기술부 국가지정연구실 사업의 지원 으로 수행되었다.

## 참고문헌

1. Choi, B. K., Kim, D. H. and Robert B Jerard, "Cspace approach to tool-path generation for die and mould machining, Computer-Aided Design, Vol. 29, No. 9, pp. 657-669, 1997.
2. Cardan, Y. and Perrin, E., 'An algorithm reducing 3D Boolean operatoins to a 2D problem: concepts and results," Computer-Aided Design, Vbl. 28, No. 4, pp. 277-287, 1996.
3. Volino, P. and Thalmann, N. M., "Efficient self-col­ lision detection on smoothly discrretized surface ani­ mations using geometrical shape regularity/5 Computer Graphics Forum (EuroGraphics Proc.), Vbl. 13, pp. 155-166, 1994.
4. Mezger, J., Kimmerle, S. and Etzmub, O., ''Progress in collision detection and response techniques for closth animation5', Proc, of the 10th Pacific Confer­ ence on Computer Graphics and Application, 2002.
5. Lau, R. W. H., Chan, O' Luk, M. and Li, F. W. B., 'A collision detection framework for deformable objects, ACM VRST'02, November 11-13, 2002.
6. Provot, X., "Collision and self-collision handling in cloth model dedicated to design garments/' Graphics Interface, pp. 177-189, May 1997.
7. Samet, H., "Applications of spatial data structures computer grapichs, image processing and GIS," Addison-We 이 ey, 1990.
8. Moiler, T., "A fasst triangle-triangle intersection test/ Journal of Graphics Tools, Vol. 2, No. 2, pp. 25-30, 1997.
9. Ruppert, J., 'A delaunay refinement algorithm for quality 2-Dimensional mesh generation," Journal of Algorithms, Vbl. 18, No. 3, pp. 548-585, May 1995.
10. Choi, B. K. and Jerard, R. B., '"Sculptured surface Machining - theory and applications", Kluwer Aca­ demic Publishers, 1998.

<!-- image -->

<!-- image -->

## 정 원 형

1998 년 숭실대학교 산업공학 학사 20 如 0 년 KAIST 산업공학 석사 2000 년 〜 현재 KAIST 산업공학 박사과정 재학

관심분야: 형상모델링, CAD/CAM, CG, Reverse engineering, Mesh generation

## 신 하 용

1985 년 서울대학교 산업공학 학사 1987 년 KAJST 산업공학 석사 199 ［년 KAl^t 산업공학 박사 1991~1993 년 LG 생산기술원 선임연구원 1993T997 년 ĵ주Ķ큐빅테크 이사 1997~20 이년 DaimlerChrysler Senior specialist

2001 년~현재 KAHST 산업공학과교수 관심분야: 형상모델링, CAD/CAM, CG, DMU, VMS, Reverse engineer­ ing, BPM

<!-- image -->

<!-- image -->

## 정 춘 석

2001 년 한양대학교 산업공학 학사 2003 년 KAIST 산업공학 석사 2003 년~현재 ĵ주Ķ DMS 연구소 연구원 관심분야: 형상모델링, CAD/CAM, CG, Reverse engineering

## 최 병 규

1973 년 서울대학교 산업공학 학사 1975 년 KAIST 산업공학 석사 1982 년 미국 Purdue 대 생산공학 박사 1982 년 〜 현재 KAIST 산업공학과 교수 관심분야: 형상모델링, CAD/CAM, CAPP, 자동화 제조시스템 모델 링 및 시뮬레이션, VMS, eBusiness, BPM