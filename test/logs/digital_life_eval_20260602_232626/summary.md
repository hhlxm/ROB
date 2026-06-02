# Digital Life Test3 Evaluation

- Generated at: 20260602_232626
- Provider: default / qwen3.5-4b
- Log dir: `/mnt/emmc/lxm/ROB/test/logs/digital_life_eval_20260602_232626`
- Completed: 84/84
- Overall: 64/84 = 0.762

## Group Accuracy

| Group | Scenario | Success | Total | Accuracy |
|---:|---|---:|---:|---:|
| 1 | 文件管理 | 2 | 4 | 0.500 |
| 2 | 相册检索 | 4 | 4 | 1.000 |
| 3 | 照片元数据 | 4 | 4 | 1.000 |
| 4 | 影音控制 | 10 | 11 | 0.909 |
| 5 | 安防事件 | 0 | 8 | 0.000 |
| 6 | 身份识别 | 5 | 7 | 0.714 |
| 7 | PDF 操作 | 6 | 6 | 1.000 |
| 8 | Word 操作 | 4 | 4 | 1.000 |
| 9 | PPT 生成 | 2 | 3 | 0.667 |
| 10 | 表格处理 | 1 | 4 | 0.250 |
| 11 | 文本总结 | 3 | 3 | 1.000 |
| 12 | PDF 元信息 | 4 | 4 | 1.000 |
| 13 | 翻译 | 4 | 4 | 1.000 |
| 14 | OCR | 3 | 4 | 0.750 |
| 15 | 文本改写 | 4 | 5 | 0.800 |
| 16 | 结构化提取 | 3 | 3 | 1.000 |
| 17 | 笔记操作 | 2 | 3 | 0.667 |
| 18 | 笔记检索 | 3 | 3 | 1.000 |

## Failure Reasons

| Count | Reason |
|---:|---|
| 7 | timeout returncode=124 |
| 3 | tool_call_count expected=1 actual=2 |
| 1 | tool expected=digital_file_manager actual=digital_pdf_document; action expected=get_properties actual=get_metadata; path expected_contains=/xx/合同.pdf actual=None |
| 1 | episode_number expected=5 actual=None |
| 1 | returncode=1 |
| 1 | area expected_contains=客厅 actual=None |
| 1 | slide_count expected=5 actual=None |
| 1 | formula expected_contains=求和 actual==SUM(A2:A10) |
| 1 | filter_condition expected_contains=金额大于 1000 actual=金额大于 10 |
| 1 | sheet_name expected_contains=['项目预算', '预算'] actual=None |
| 1 | max_chars expected=200 actual=None |
| 1 | note_id expected_contains=note_mock_001 actual=note_mock_01 |

## Failure Samples

- G1.1 `看下 /xx/合同.pdf 的属性`: tool expected=digital_file_manager actual=digital_pdf_document; action expected=get_properties actual=get_metadata; path expected_contains=/xx/合同.pdf actual=None (actual=digital_pdf_document, first_slot=get_metadata, session=501e9ba3-e467-4fe2-99d0-0c242e29d5aa)
- G1.4 `这个文件多大`: timeout returncode=124 (actual=digital_file_manager,digital_file_manager,digital_file_manager...(+23), first_slot=get_properties, session=e22bc35f-9feb-4f0b-83eb-5e08279a6894)
- G4.7 `跳到第5集`: episode_number expected=5 actual=None (actual=digital_media_control, first_slot=jump_episode, session=ace942f0-1c76-4db5-af5b-ff28d4c74b52)
- G5.1 `门口有人吗`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+23), first_slot=person, session=c63442bc-24c5-4a1e-9f50-b1bb2163ca09)
- G5.2 `刚刚院子有人没`: returncode=1 (actual=none, first_slot=, session=)
- G5.3 `客厅有动静吗`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+22), first_slot=motion, session=b5b9fe6d-6df1-4d78-863a-8c65e1e19b5b)
- G5.4 `今天有车进出过吗`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+20), first_slot=vehicle, session=21f722de-1937-4b19-b461-a1ff3a6570c5)
- G5.5 `下午门口有快递吗`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+22), first_slot=package, session=35a7540a-54d4-4583-a482-5d8d8e1038fb)
- G5.6 `今晚有人来过吗`: tool_call_count expected=1 actual=2 (actual=digital_security_event_query,digital_security_event_query, first_slot=person, session=49e0f118-0ce5-4dc3-9ee3-3edf3301376d)
- G5.7 `宝宝房间有人进去吗`: tool_call_count expected=1 actual=2 (actual=digital_security_event_query,digital_security_event_query, first_slot=entry, session=9817e322-701b-4331-b2fd-c9ec6d43c44e)
- G5.8 `刚才阳台有动静没`: timeout returncode=124 (actual=digital_security_event_query,digital_security_event_query,digital_security_event_query...(+22), first_slot=motion, session=59fd5fde-f02c-4210-9d02-877e7badc93e)
- G6.2 `客厅有陌生人吗`: area expected_contains=客厅 actual=None (actual=digital_security_identity_history, first_slot=stranger, session=0f28cf43-87a7-4099-b49a-7c02f1a49d29)
- G6.5 `送快递的来过吗`: tool_call_count expected=1 actual=2 (actual=digital_security_identity_history,digital_security_identity_history, first_slot=送快递的, session=fd58be18-173c-4c0b-aba3-fe35fd6b049a)
- G9.1 `按这个大纲做个 5 页的 PPT`: slide_count expected=5 actual=None (actual=digital_ppt_generation, first_slot=create_from_outline, session=b470d0f5-2e2b-42c2-aade-56d106487774)
- G10.1 `在这个 xlsx 加一列求和公式`: formula expected_contains=求和 actual==SUM(A2:A10) (actual=digital_spreadsheet, first_slot=add_formula_column, session=bfc0a8fd-f002-47a4-9974-646d211e8cef)
