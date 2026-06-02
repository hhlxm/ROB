# Digital Life Test3 Evaluation

- Generated at: 20260602_221131
- Provider: default / qwen3.5-4b
- Log dir: `/mnt/emmc/lxm/ROB/test/logs/digital_life_eval_20260602_221131`
- Completed: 84/84
- Overall: 53/84 = 0.631

## Group Accuracy

| Group | Scenario | Success | Total | Accuracy |
|---:|---|---:|---:|---:|
| 1 | 文件管理 | 2 | 4 | 0.500 |
| 2 | 相册检索 | 4 | 4 | 1.000 |
| 3 | 照片元数据 | 4 | 4 | 1.000 |
| 4 | 影音控制 | 10 | 11 | 0.909 |
| 5 | 安防事件 | 0 | 8 | 0.000 |
| 6 | 身份识别 | 0 | 7 | 0.000 |
| 7 | PDF 操作 | 6 | 6 | 1.000 |
| 8 | Word 操作 | 4 | 4 | 1.000 |
| 9 | PPT 生成 | 1 | 3 | 0.333 |
| 10 | 表格处理 | 1 | 4 | 0.250 |
| 11 | 文本总结 | 3 | 3 | 1.000 |
| 12 | PDF 元信息 | 4 | 4 | 1.000 |
| 13 | 翻译 | 3 | 4 | 0.750 |
| 14 | OCR | 3 | 4 | 0.750 |
| 15 | 文本改写 | 3 | 5 | 0.600 |
| 16 | 结构化提取 | 0 | 3 | 0.000 |
| 17 | 笔记操作 | 2 | 3 | 0.667 |
| 18 | 笔记检索 | 3 | 3 | 1.000 |

## Failure Reasons

| Count | Reason |
|---:|---|
| 14 | timeout returncode=124 |
| 4 | tool_call_count expected=1 actual=2 |
| 1 | tool expected=digital_file_manager actual=digital_pdf_document; action expected=get_properties actual=get_metadata; path expected_contains=/xx/合同.pdf actual=None |
| 1 | action expected=resume actual=resume_last |
| 1 | slide_count expected=5 actual=None |
| 1 | formula expected_contains=求和 actual==SUM(A2:A10) |
| 1 | filter_condition expected_contains=金额大于 1000 actual=金额大于 10 |
| 1 | sheet_name expected_contains=['项目预算', '预算'] actual=None |
| 1 | target_language expected_contains_any=['中文', 'Chinese'] actual=None |
| 1 | action expected=rewrite actual=compress; style expected_contains_any=['简洁'] actual=None |
| 1 | max_chars expected=200 actual=None |
| 1 | document_type expected_one_of=['invoice', 'unknown', 'image'] actual=None |
| 1 | document_type expected_one_of=['contract', 'pdf', 'unknown'] actual=None |
| 1 | document_type expected_one_of=['receipt', 'ticket', 'unknown', 'image'] actual=None; fields expected_contains=商户 actual=['merchant'] |
| 1 | note_id expected_contains=note_mock_001 actual=note_mock_01 |

## Failure Samples

- G1.1 `看下 /xx/合同.pdf 的属性`: tool expected=digital_file_manager actual=digital_pdf_document; action expected=get_properties actual=get_metadata; path expected_contains=/xx/合同.pdf actual=None (actual=digital_pdf_document, action=get_metadata, session=ed799fa4-462d-434b-a661-d1410b68451f)
- G1.4 `这个文件多大`: tool_call_count expected=1 actual=2 (actual=digital_file_manager,digital_file_manager, action=get_properties, session=1bc6f186-019b-4974-b006-708696286385)
- G4.4 `继续播放`: action expected=resume actual=resume_last (actual=digital_media_control, action=resume_last, session=952e2dc5-3a51-4a50-970f-b5a32e6b4f99)
- G5.1 `门口有人吗`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+66), action=, session=5e737e96-8f0c-4bbd-b511-ce9b452ff172)
- G5.2 `刚刚院子有人没`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+63), action=, session=dd55b943-b04f-4727-bdcd-162c5241e711)
- G5.3 `客厅有动静吗`: tool_call_count expected=1 actual=2 (actual=digital_security_event_query,digital_security_event_query, action=, session=05b6beac-8e7c-4470-b7eb-eafb235101a0)
- G5.4 `今天有车进出过吗`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+13), action=, session=f646b341-9632-472d-b3d0-4f9c64689026)
- G5.5 `下午门口有快递吗`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+14), action=, session=d84582bc-f1da-4816-bbbe-4ecfa90208b7)
- G5.6 `今晚有人来过吗`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+14), action=, session=4e7ad6f6-8b89-4b2a-801c-e5d74023c25a)
- G5.7 `宝宝房间有人进去吗`: tool_call_count expected=1 actual=2 (actual=digital_security_event_query,digital_security_event_query, action=, session=399b761d-6d37-41ec-b075-2f3a084a4604)
- G5.8 `刚才阳台有动静没`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+13), action=, session=d0e20477-98e2-4ec2-bc65-b133042a0c4a)
- G6.1 `爸爸回来了吗`: timeout returncode=124 (actual=digital_security_identity_recognition,digital_security_identity_recognition,digital_security_identity_recognition...(+14), action=, session=7bddde4f-af46-4995-8a21-40c996f83225)
- G6.2 `客厅有陌生人吗`: timeout returncode=124 (actual=digital_security_identity_recognition,digital_security_identity_recognition,digital_security_identity_recognition...(+14), action=, session=59c57862-83c4-4bbf-a8f6-0731303fcc73)
- G6.3 `张三今天来过没`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+14), action=, session=0ff99a31-f06f-45dc-9b6f-e0fff6c44e74)
- G6.4 `妈妈到家了没`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+13), action=, session=1991abb6-4994-4efb-b34e-4fd6b2037122)
