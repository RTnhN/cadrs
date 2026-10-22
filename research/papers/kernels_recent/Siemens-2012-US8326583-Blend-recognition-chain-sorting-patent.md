## (12) United States Patent Yu et al.

US008326583B2

<!-- image -->

- (10) Patent No.:

US 8,326,583 B2

- (45) Date of Patent:

Dec. 4, 2012

- (54) CAD SYSTEM AND METHOD FOR BLEND RECOGNITION AND CHAIN SORTING IN A BREP MODEL

- (75) Inventors: Feng Yu, Irvine, CA (US); Yu (Peggy)

Shanghai (CN); Eric Mawby, Windham, Xia, Shanghai (CN); Ying (Tina) Lin, NH (US)

- (73)  Assignee: Siemens Product Lifecycle Management Software Inc., Plano, TX

(US)

- (*） Notice:

Subject to any disclaimer, the term of this patent is extended or adjusted under 35 U.S.C. 154(b) by 472 days.

- (21)Appl. No.: 12/617,184

- (22) Filed:

Nov. 12, 2009

- (65) Prior Publication Data US 2010/0305906 A1 Dec. 2, 2010

## Related U.S. Application Data

- (60) Provisional application No. 61/182,462, filed on May 29, 2009.

- (51) Int. Cl. G06F 17/50 (2006.01)

- (52) U.S. Cl. ... 703/1

- (58) Field of Classification Search 703/1 Scc application file for complete search history.

## (56) References Cited

## U.S. PATENT DOCUMENTS

| 6,133,922 A *10/2000   |         | ）Opitz                           | 345/420   |
|------------------------|---------|----------------------------------|-----------|
| 7,236,168 B2           | 6/2007  | Venkataraman et al.              |           |
| 7,830,377 B1 *         |         | 11/2010 0Desimone et al. +-+-+-+ | 345/420   |
| 2002/0180729 A1*       | 12/2002 | Venkataraman et al.... 345/420   |           |
| 2004/0257362 A1        | 12/2004 | Venkataraman et al.              |           |
| 2007/0027665 A1*       | 2/2007  | 7Maille et al.                   | 703/6     |
| 2008/0174599 A1*       | 7/2008  | Rothstein et al.                 | 345/420   |
| 2010/0121626 A1*       |         | 5/2010Montana et al. ......      | 703/6     |

## FOREIGN PATENT DOCUMENTS

LP

1710720 Al

10/2006

WO

WO 2008088369 A1

OTHER PUBLICATIONS

7/2008

Xiufen Cui, et al., "An Efficient Algorithm for Recognizing and Suppressing Blend Features": p. 421-428. Venkataraman, Sashikuma1, et al., "SM38: Removal of Blends fron

Boundary Representation Models", 12 pages.

* cited by examiner

Primary Examiner —Kandasamy Thangavelu

Assistant Examiner — Nithya Janakiraman

## (57) ABSTRACT

A CAD system, method, and computer readable medium. A method includes loading a CAD object including a plurality of faces, and identifying a plurality of candidate faces from the plurality of faces. The candidate faces can include candidate blcnds. The mcthod includcs combining multiple ones of the candidate faces into a candidate blend chain, including at least one candidate blend. The method includes replacing the candidate blends with accurate blends to produce a corrected blend chain, and storing the corrected blend chain with the accurate blends as part of the CAD objcct.

## 20 Claims, 5 Drawing Sheets

<!-- image -->

<!-- image -->

<!-- image -->

Figure 2A

Figure 2B

<!-- image -->

Figure 3

<!-- image -->

<!-- image -->

Figure 6

<!-- image -->

Figure 7

<!-- image -->

## CADSYSTEMANDMETHODFORBLEND RECOGNITION AND CHAIN SORTING IN A BREP MODEL

tures for carrying out the same purposcs of the present disclosure. Those skilled in the art will also realize that such equivalent constructions do not depart from the spirit and scope of the disclosure in its broadest form.

CROSS-REFERENCE TO OTHER APPLICATION 5 Before undertaking the DETAILED DESCRIPTION below, it may be advantageous to set forth definitions of This application claims priority from U.S. Provisional certain words or phrases uscd throughout this patent docuPatent Application 61/182,462, filed May 29, 2009, which is ment: the terms "included and "comprise,' as well as derivahereby incorporated by reference. tives thereof, mean inelusion without limitation; the term *"or"* 10 is inclusive, meaning and/or; the phrases "associated with" TECHNICAL FIELD and "associated therewith," as well as derivatives thereof, may mean to include, be included within, interconnect with, The present disclosure is directed, in general, to computercontain, be contained within, connect to or with, couple to or aided design, visualization, and manufacturing systems, and with, be communicable with, cooperate with, interleave, juxsimilar systems. referred to herein individually and collec15 lapose, be proximate 1o, be bound to or with, have, have a tively as *CAD systems.*" property of, or the like; and the term "controller" means any device, system or par thereof that controls at least one operaBACKGROUND OF THE DISCLOSURE tion, whether such a device is implemented in hardware, firmware, software or some combination of at least two of the In CAD systems, a *blend" is a surface used to smoothly 20 same. It should be noted that the functionality associated with connect two or more othcr surfaces. any particular controller may be centralized or distributed, whether locally orremotely. Definitions for cerlain words and SUMMARY OF THE DISCLOSURE phrases are provided throughout this patent document, and those of ordinary skill in the art will understand that such definitions apply in many, if not most, instances to prior as well as future uses of such defined words and phrases. While some terms may include a wide variety of embodiments, the appended claims may cxprcssly limit thesc terms to specific embodiments.

## BRIEF DESCRIPTION OF THE DRAWINGS

For a more complctc undcrstanding of the prescnt disclosure, and the advantages thereof, reference is now made to the 35 following descriptions taken in conjunction with the accompanying drawings, wherein like numbers designate like objects, and in which:

Various embodiments include CAD systems, methods, and 25 computer readable mediums. In one embodiment, a method. includes loading a CAD object including a plurality of faces, and identifying a plurality of candidate faces from the plurality of faces. The candidate faces can include candidate blends. The method includes combining multiple ones of the candi- 30 date faces into a candidate blend chain, including at least one candidate blend. The method includes replacing the candidate blends with accurate blends to produce a corrected blend chain, and storing the corrected blend chain with the accurate blends as part of the CAD object.

FlGS. 1 through 7, discussed below, and the various disclosure in this patent document are by way of illustration only and should not be construed in any way to limit the scope of the disclosure. Those skilled in the art will understand that the principles of the present disclosure may be implemented 60 in any suitably arranged device. The numerous innovative teachings of the present application will be described with reference to exemplary non-limiting embodiments.

In some embodiments, each candidate face is one of a B-surface, partial cylinder, partial torus, partial sphere, blend, non-periodic extruded, or non-periodic revolved. In some FIG. 1 depicts a block diagram of a data processing system embodiments, an accurate blend is a face having a cross in which an embodiment can be implemented; section that is circular and of constanl radius. In some 40 FIGS. 2A and 2B depict an examplc of blend geomctry embodiments, the CAD system identifies blend radii for each imported from a foreign source; of the candidate blends, and groups the candidate faces FlG. 3 depicts an example corresponding to that of FIG. according to the blend radi. In some embodiments, groups of 2A: e FlG. 4 shows another example of blend geometry imported chain. In some embodiments, the CAD system identifies al 45 from a foreign sourcc; least one blend-like face in a candidate chain, finds a radius FIG. 5 shown an example corresponding to that of FIG. 4; and construction face of each of the blend-like faces, and. FIG. 6 depicts a flowchart of an overall process in accorrecognizes the blend-like face as a blend. In some embodidance with disclosed cmbodiments; and ments, the CAl system uses radi and construction faces for FIG. 7 illustrates four corner blend types that can be proeach of the faces in the correcled blend chain to apply a blend 50 cessed using the tcchniqucs discloscd hercin. process to the CAD object. In some embodiments, the CAD) system performs a cormer blends handling process to recogDETAILEI DESCRIPTION nize and correct at least one candidate blend that forms a corner between at least three faces. In some embodiments, the CAD syslem calegorizes a candidate blend as one ofa triangle 5  embodimenis used to describe the principles of the present corner face, a triangle-like corner, a variable-radius corner face, and a four-sided corner face. In some embodiments, the candidate blend that forms a corner is combined in a single chain with its neighboring faces.

One synchronous modeling approach for design change emphasizes modifying the curent state of a model without y 65 regard for how the model was constructed. Synchronous modeling is expected to perform in a source-independent a  s  n  n

The foregoing has outlined rather broadly the fealures and. technical advantages of the present disclosure so that those skilled in the art may better understand the detailed description that follows. Additional features and advantages of the disclosure will be described hereinafter that forrm the subjecl of the claims. Those skilled in the art will appreciate that they may readily use the conception and the specific embodiment disclosed as a basis for modifying or designing other struc- forcign. Model gcomctry imported from forcign sources often includes sub-optimal conditions that hinder the success Of synchronous modeling operations. Sub-optimal conditions need to be rectified in order to modify the geometry in a credible manner.

permits multiple display windows to be prescnted in the graphical user interface simultaneously, with each display window providing an interface to a different application or to a different instance of the same application. A cursor in the graphical user interface may be manipulated by a user through the pointing device. The position of thc cursor may bc changed and/or an event, such as clicking a mouse button, generated to actuate a desired responsc.

LAN/WAN/Wireless adapter 112 can be connected to a 115 network 130 (not a part of data processing system 100), which can be any public or private data processing system network Or combination of networks, as known to those of skill in the art, including the Intcrnet. Data processing systcm 100 can communicate over network 130 with server system 140, 20 which is also not part of data processing system 100, but can be implemented, for example, as a separate dala processing system 100.

A "reblend" opcration within synchronous modeling is used when a design face is changed. In a reblend, the surrounding blend faces are recomputed (reblended) to accommodate the change applied to the design face. In order to One of various commercial operaling systems. such as a properly reblend, the CAD system must be able to recognize 1o version of Microsoft WindowsTM, a product of Microsoft the blcnd faccs as blcnds. Typically, blends crcated nativcly or Corporation located in Redmond, Wash. may be cmployed if within a completely-compatible CAD system are sufficiently suitably modified. The operating system is modified or creaccurate to recognizc as blends. However, with imported ated in accordance with the prescnt disclosure as described. geometry, the blend faces are frequenily not accurate enough behaved."*

In a CAD system as discussed above, one condition to recognize a face as a blend is tha1 the face must match the shape of a rolling ball. That is, the cross section of the blend face must be circular and conslant radius. In most cases, a blend added natively or created in a completely-compatible CAD system matches the rolling ball form. A blend imported from a foreign source often does not. The foreign system may not impose the same accuracy ideals or the accuracy of a face may have been altered during translation, as the model is cxported from one CAD system and into another. In many mately as a boundary representation (Brep) surface (b-sur35 face).

A blend-like face might look like a blend but may not bchave likc a blend. That is, it fails to rcblend becausc it is not a blend. If a blend-like face fails to reblend, the techniques described herein can replace it. To replace a blend-like face, the user can select the blend-like face, and the system will replace the blend-like face with a blend face using the methods described in detail below. Various embodiments disclosed. herein can be used for blend recognition and processing.

FIG. 1 depicts a block diagram ofa data processing system in which an embodiment can be implemented, for example as a CAD system for performing the blend-related processes described herein. The data processing system depicted includes a processor 102 connected to a level two cache/ bridge 104, which is connected in turn to a local system bus 30 106. Local system bus 106 may be, for example, a peripheral component intcrconneet (PCI) architecture bus. Also connected to local system bus in the depicted example are a main memory 108 and a graphics adapter 110. The graphics adapter 110 may be connected to display 111.

Even if an imported blend is represented as a b-surface, some CAD systems will recognize it as a blend if it matches rolling ball form within tolerance. However, a percentage of imported blends deviate from the rolling ball form beyond a r 40 reasonable tolerance. Another percentage is well beyond the shape of a rolling ball and is often not well-behaved, and not even visually recognizable as blends. In various embodiments, the tolerance discussed herein can be either an absolute value or a value relative to the size of the candidate blend face. A user can adjust the value based on the accuracy the user wants the model to have. The techniques described herein work particularly well for a tolerance smaller than %/10 of the candidate face size.

Also connected to I/O bus 1l6 in the example shown is audio adapter 124, to which speakers (not shown) may be connected for playing sounds. Keyboard/mouse adapter 118 provides a connection for a pointing device (not shown), such as a mouse, trackball, trackpointer, etc.

Other peripherals, such as local area network (LAN)/Wide Area Network/Wircless (e.g. WiFi) adapter 112, may also be connected to local system bus 106. Expansion bus interface 114 connects local system bus 106 to input/output (I/O) bus 116. I/O bus 116 is connected to keyboard/mouse adapter 118, disk controller 120, and I/O adapter 122. Disk controller 120 can be connected to a storage 126, which can be any suitable machine usable or machine readable storage medium, including but not limited to nonvolatile. hard-coded type mediums such as read only memories (ROMs) or eras- 45 able, electrically programmable read only memories (EEPROMs), magnetic tape storage, and user-recordable type mediums such as floppy disks, hard disk drives and compact disk read only memories (CD-ROMs) or digital versatile disks (DVDs), and other known optical, electrical, or mag- 50 netic storage devices.

Those of ordinary skill in the art will appreciate that the hardware depicted in FIG. 1 may vary for particular implementations. For example. other peripheral devices, such as an optical disk drive and the like, also may be used in addition or 60 in place of the hardware depicted. The depicted example is provided for the purpose of explanation only and is not meant to imply architectural limitations with respect to the present disclosure.

Disclosed embodiments include CAD systems and methods that recognize the chain of blend-like connecting faces for the purpose of replacing those faces with proper rolling-ball blends. For example the embodiments disclosed herein allow - 65 the system to recognize the blend chain of faces 205/220/210/ 225/215, and replace connecting faces 220/225 with rolling ball blends. FIG. 3 depicts an example corresponding to that

FlG. 2A depicts an example of blend geometry imported from a foreign source. Not knowing the definition of faces from the foreign source, the CAD system can assume that the end faces 205/210/215 and connecting faces 220/225 compose a "blend chain,' a set of faces intended to be blended to form a smooth surface. FIG. 2B shows a close up of a portion 1 55 of FIG. 2A. Here, the close-up image of the cormer al connecting face 225 reveals that the blends do not match rolling ball form. The blends with end faces 210 and 215 "flair" where they meet the blends of connecting face 225, and the blends of connecting face 225 deviate substantially from the 0 torus shape that would be expecled from a proper blend.

A data processing system in accordance with an embodiment of the present disclosure includes an operating system employing a graphical user interface. The operating system

0f FIGi. 2A, where accuratc blcnds 320/325 have becn put in place of corresponding connecting faces 220/225.

The CAD system analyzes the object model to identify candidate faces that can be part of a blend chain (step 610), including identifying one or more connecting faces blends in the blend chain.

There are very few cases where a blend face exists in isolation of other blends. In a realistic model a blend is almost always a member of a chain of blends, and the blend chain is s50 typically a member of a network of intcr-depcndent blends. As such, reblending a blend means the blend chain and its dependent blends participate in an extended reblend operation.

proccssing systcm, for cxample ovcr a nctwork, rccciving it from a user input, or other means recognized by those of skill in the art.

Imported blends represent challenges to successful reblend Of a network. The challenge is compounded by the number of blends in the nelwork. A blend chain by definition shares the same blend radius. Inaccuracy of radi of approximate b-surface blends may make it impossible to recognize a blend chain.

A corner blend is particularly challenging to recognize as a blend. There are various representations of corner blend which vary by the number of edges composing the corner, the convexity of the edges, and the radius of the edges. Corners 20 are members of blend chain and necessary to the success of reblending a network of blends.

In this step, the CAD system identifies a chain of connected candidate faces, that can include connecting faces including 10 accurate blends, tolerant blends, and blend-like faces, as faces for a blend chain, and identifies thc candidate blends in thc chain. For each selected candidate face, the CAD system performs a blend recognition process with a rcasonable predetermined tolerance. If the face is recognized as an accurate 15 1 blend or tolerant blend, the CAD system also performs the blend recognition process on each candidate face that connects to this candidate face. All the accurate and tolerant blends connectcd to thc sclccted candidates arc identified this way.

FIG. 4 shows another example of blend geometry imported from a foreign source. The three blends 405/410/415 entering the face 420 of the corner are all different radius. The defini- 25 radius is to be used for each group, and can be determined in tion of the corner from the foreign source is a variable radius transition between the two smaller radii, rolling along the larger radius. The variable radius transition does not match rolling ball form.

As the CAD system recognizes that candidate faces and blends, the CAD system also identifies the blend radi for each of the identified blends. The CAD system puts the collected candidates into groups of the same radi modulo the devialion associated with the recognition tolerance. One a way such that it can be reused on replay of the operation.

Construction faces of each accurate and tolerant blend can

The CAD system designates each connected set of faces in a group of radius as a blend chain. For each of these chains, the CAD system can find the dependent blends and includes e 30 them in the chain to process together. The CAD system can merge multiple chains if one blend in a chain is a dependent of another chain or two chains are connected by a dependent blend.

The CAD system thien combines the selected candidates into chains (step 615). In this process, the CAD system also incorporates blend-like candidates into a chain. After the FIG. 6 depicts a flowchart of an overall process in accorchains of accurate blends and tolerant blends are determined, dance with disclosed embodiments. Following are the par- 40 the remaining selected candidates are the blend-like faces. ticular definitions generally used in the description below, For ofthe candidate blend-like faces, the CAD system find the although those of skill in the art will recognize when the accurate blends, tolerant blends and candidate blend-like context requires a different use: faces that are connected to it. Based on the type of faces Candidate: B-surface, partial cylinder, partial torus, partial found, the CAD system propagates the recognition along the sphere, blend, non-periodic exiruded, or non-periodic 45 connected candidates to find the radius and construction faces revolved. of the blend-like face so that they can be recognized as a blend.

The CAl systems and methods described herein recognize the face 420 at the corner, determine the edge blends that compose the corner, their radius and convexity, and determine an order thal will produce rolling ball blends at the corner, all for the purpose of replacing face 420 with rolling ball blends. FIG. 5 shown an example corresponding to that of FIG. 4, 35 also be determined in the process this step. where the corner face 420 has been recognized by the CAD system and replaced with accurate rolling ball blends as faces 530/535.

Accurate Blend: A candidate that is accurate to the rolling ball form as described herein, or other otherwise an accurate blend by the requirements of the CAD system. In some embodiments, this can include blends that can be identified by 50 a minimum tolerance.

For example, if two connected candidates are found thiat are opposite to each other, these two candidates are treated as Ones in the same chain. If a commecled candidate is an accurate blend or tolerant blend, the CAD system obtains a radius from it. Otherwise, the CAl) system performs the same process on the connected candidate until the radius is obtained. The other connected faces, that is, non-candidate faces, are treated as

The CAD system put the selected blend-like face into the chain where the radius is found, and can merge the chains if selected blend-like faces connect them.

Tolerant Blend: A candidate that is not an accurate blend. but is recognized as blend with a reasonable tolerance. That is, within reasonable tolerance of the rolling ball form. In some embodiments, this can include blends that can be iden- 55 construclion faces. ance.

Blend-Like: A candidate that is not recognized as blend with a reasonable tolerance. In some embodiments, this can This provides a significant technical advantage over other include blends that are Bsurf and swept faces that cannot be 60 systems, in which blend-like candidates are nol recognized as identified even with a maximum tolerance. Further, in some blends. The processes described herein aggressively identify embodiments, blend-like faces are all processed as corners, and process the blend-like faces to participate in a blend and other faces can also be found when identifying corners. chain.

According to at least one disclosed process, the CAD sysThe CAl) system performs a corner blends handling protem loads a CAD object model including a plurality of faces 65 cess (step 620). In this step, the CAD system can recognize (step 605). Loading, in this context, can include loading the and correct various types of corner blends. Most corner canobject model from a storage, receiving it from another data didates in imported parts are blend-like in that they are not immcdiatcly recognizablc as blends. The corners can be categorized into many types in terms of the connection to adjacent faces, the relative radius sizes of adjacent edge blends and the convexity of the cormer. This step is optional, and if performed, can be performed concurrently with other steps.

The triangle corner category includes a candidate face that is adjacent to three independent blends. A triangle-like corner calegory includes a candidate face adjacent to three dependent blends. A variable-radius corncr category includcs a candidate face that is adjacent to three independent blends with three variable radiuses and one non-blend face. A foursided corner category includes a candidate face that is a branch face adjacent to three blends and one non-blend face.

Those skilled in the art will recognize that, for simplicity and clarity, the full structure and operation of all data processing systems suitable for use with thc prcsent disclosure is not being depicted or described herein. Instead, only so much of a data processing system as is unique to the present disclosure or necessary for an understanding of the presen1 disclosure is depicted and described. The remainder ofthe construcA mn s ssnnd p o o n conform to any of the various current implementations and practices known in the art.

FIG. 7 illustrates four corncr blcnd typcs that can bc processed using the techniques disclosed herein. These types can be further grouped, in some embodiments, into 4 large category sets that are handled separately in this step.

These figures illustrate some of these categories, such as a triangle cormer face 705, triangle-like corner with faces 710/ 715, a variable-radius corner face 720, and a four-sided corner face 725.

Other embodiments can also include a connected corner category that includes a candidate face that is a blend-like face adjacent to two blends in a chain and two non-blend faces which connects a "broken" chain. Of course, other faces can be non-corner faces.

Returning to step 620, for each selected candidate corner face, the CAD system determines to which of the corner categories the candidate belongs (as described with relation to FIG. 7), and processes the candidate based on the characteristics of the corner category to obtain the radius and construction face information.

mentcd by a computer-rcadable medium cncodcd with instructions that, when executed, cause a data processing system to perform such steps.

108

30

It is important to note that while the disclosure includes a 15 description in the context of a fully functional system, those skilled in the art will appreciate that at least portions of the mechanism of the present disclosure are capable of being distributed in the form of a instructions containcd within a machine-usable, computer-usable, or computer-readable 20 medium in any of a variety of forms, and that the present disclosure applies equally regardless of the particular type of instruction or signal bearing medium or storage medium utilized to actually carry out the distribution. Examples of machine usable/readable or compuler usable/readable medis 25 ums include: nonvolatile, hard-coded type mediums such as read only memories (ROMs) or erasable, electrically programmable read only memories (EEPROMs), and user-recordable type mediums such as floppy disks. hard disk drives and compact disk read only memories (CD-ROMs) or digital versatile disks (DVDs).

For example, if the candidate blend is identified as belong35 ing lo a triangle category 705, the three adjacent faces are accurate blends, tolerant blends or selected blend-like faces. They are the construction faces of the cormer. The respective radi of them are used to as the radius data for the corner. Proccssing is similar for cach of the other corner categories; each corner category has an associated corner category defimition that defines the number of adjacent faces and their radii that should be used for processing the corner blend.

The CAI) system replaces the candidate blends in the identified chains with complete, accurate blends (step 625); the chains can then be regarded as corrected blend chains. In this process, each of the candidates in each chain is replaced with 50 accurate blends, created according to the radi and construction faces found in a previous step and according to the radii Of neighboring faces. After all candidates are processed and. put into independent chains, the CAD system replaces them with accurale blends, and stores the new surfaces and blends 55 as part of the CAD object model (step 630).

Although an exemplary embodiment of the present disclosure has been described in detail, those skilled in the art will understand thal various changes, substitutions, variations, a    r  s departing from the spirit and scope of the disclosure in its broadest form.

None of the description in the present application should be read as implying that any particular element, step, or function is an essential element which must be included in the claim ; 40 scope: the scope of patented subject matler is defined only by the allowed claims. Moreover, none of these claims are intended to invoke paragraph six of 35 USC §112 unless the exact words "means for? are followed by a participle.

What is claimed is:

- loading a CAD object in a CAD system, the CAD object including a plurality of faces;

1. A method for use in a CAD system, comprising:

- identifying. by the CAD system, a plurality of candidate faces from the plurality of faces, the candidate faces including candidate blends, multiple ones of the candidate blends having different blend radii;
- combining. by the CAD system, multiple ones of the candidate faces into a candidate blend chain. including at least one candidale blend;
- replacing, by the CAD system, the candidate blends with accurate blends to produce a corrected blend chain, the accurate blends having the same blend radii; and

2. The method of claim 1, wherein each candidate face is one of a B-surface, partial cylinder, partial torus, partial sphere, blend, non-periodic extruded, or non-periodic revolved.

- storing the corrected blend chain with the accurate blends in the CAD system as parl of the CAD object.

Those of skill in the art will recognize that not all embodis s   d  on ss  an s can be performed in a different order or concurrently unless specifically indicated otherwise. The process of FIG. 6 can be 65   3. The method of claim 1, wherein an accurate blend is a performed by a CAD system, implemented in a CAD system face having a cross section that is circular and of constant parlicularly configured to perform such steps. and impleradius.

When a corner is processed, its adjacent faces that are also candidates are processed together. The chains of the adjacen1 45 faces and the corner are merged into a single chain.

For each chain, the CAD system uses the collected radii and construction faces as the input data to apply a blend process to blend all members together. The new faces are recognized as accurale blends, and can thereafter be 60 re-blended when construction face change.

4. Thc mcthod of claim 1, whercin the CAD systcm identifies blend radi for each of the candidate blends, and groups the candidate faces according to the blend radii.

5. The method of claim 1, wherein groups of connected. candidate faces are combined into a candidate chain.

6. Thc mcthod of claim 1, whercin the CAD systcm identifies at least one blend-like face in a candidate chain, finds a radius and construction face of each of the blend-like faces, and recognizes the blend-like face as a blend.

7. The method of claim 1, wherein the CAD system uses radii and construction faces for cach of the faces in the correcled blend chain 1o apply a blend process to the CAD object.

8. The method of claim 1, whcrein thc CAD system performs a corner blends handling process to recognize and correct at least one candidate blend that forms a corner between at least three faces.

10. The method of claim 8, wherein the at least one candidate blend that forms a corner between at least three faces is combined in a single chain with the at least three faces.

10

1.5

9. The method of claim 8, wherein the CAD system cat18. The CAD system of claim 17, wherein the CA1) system cgorizes at least one candidate blend as one of a triangle categorizes at least one candidate blend as one of a triangle corner face, a triangle-like corner, a variable-radius corner 20 corner face, a triangle-like corner, a variable-radius corner face, and a four-sided corner face. face. and a four-sided cormer face.

13. The CAD systcm of claim 11, whercin an accurate blend is a face having a cross section that is circular and of constant radius.

15. The CAD system of claim 11, wherein the CAD system identifies at least one blend-like face in a candidate chain, finds a radius and construction face of cach of the blend-like faces, and recognizes the blend-like face as a blend.

14. The CAD system ofclaim 11, wherein the CAD system identifies blend radi for eachh of the candidate blends, and groups the candidate faces according to the blend radii.

16. The CAD) system of claim 11, wherein the CAD system uses radi and construction faces for each of the faces in the corrected blend chain to apply a blend process to the CAI object.

17. The CAD) system of claim 11, wherein the CAD system performs a corner blends handling process to recognize and corrcct at least onc candidatc blend that forms a corner between at least three faces.

19. The CAD system of claim 17, wherein the at least one candidate blend that forms a corner between at least three faces is combined in a single chain with thie at least three

11. A CAI system comprising a processor and accessible 25 faces. memory, the CAl) system configured to perform the steps of: loading a CAD object including a plurality of faces:

loading a CAD object including a plurality of faces;

20. A non-transitory computer-readable medium encoded with computer-executable instructions that, when executed, cause a data processing system to perform the steps of:

- identifying a plurality of candidate faces from the plurality of faces, the candidate faces including candidate blends, multiple ones of the candidate blends having different blend radii:
- combining multiple ones of the candidate faces into a candidate blend chain, including at least one candidate blend;
- storing the corrected blend chain with the accurate blends as part of the CAD object.
- replacing the candidate blends with accurate blends to produce a corrected blend chain, the accurate blends having the same blend radii; and
- identifying a plurality of candidate faces from the plurality of faces, the candidate faces including candidate blends, multiple ones of the candidate blends having different blcnd radi;
- combining multiple ones of the candidate faces into a candidate blend chain, including at least one candidate blend;
- replacing thc candidate blends with accurate blends to produce a corrected blend chain; and
- storing the corrected blend chain with the accurate blends as part of the CAD object.

12. The CAD systcm of claim 11, wherein cach candidate face is one of a B-surface, partial cylinder, partial torus, partial sphere, blend, non-periodic extruded, or non-periodic rcvolved.

30

35

40