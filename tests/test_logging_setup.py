import logging
import logging.handlers
import time
import unittest

from invoice_validator import logging_setup


class TestConfigureLogging(unittest.TestCase):
    def tearDown(self):
        # Undo module-level state so other tests / runs aren't affected.
        root = logging.getLogger()
        for handler in list(root.handlers):
            if isinstance(handler, logging.handlers.RotatingFileHandler):
                root.removeHandler(handler)
                handler.close()
        logging_setup._configured = False

    def test_creates_log_file_and_writes_messages(self):
        log_path = logging_setup.configure_logging()

        self.assertTrue(log_path.exists())
        logging.getLogger(__name__).info("hello from test")

        with open(log_path, encoding="utf-8") as f:
            contents = f.read()
        self.assertIn("hello from test", contents)

    def test_is_idempotent(self):
        logging_setup.configure_logging()
        logging_setup.configure_logging()

        root = logging.getLogger()
        file_handlers = [h for h in root.handlers if isinstance(h, logging.handlers.RotatingFileHandler)]
        self.assertEqual(len(file_handlers), 1)


class TestResourceHeartbeat(unittest.TestCase):
    def test_logs_periodically_until_stopped(self):
        log_path = logging_setup.configure_logging()
        logger = logging.getLogger(__name__)

        with logging_setup.ResourceHeartbeat(logger, interval=0.2):
            time.sleep(0.5)

        with open(log_path, encoding="utf-8") as f:
            hits = f.read().count("Resource check:")
        # One immediate reading on entry, plus at least one tick during the sleep.
        self.assertGreaterEqual(hits, 2)

    def tearDown(self):
        root = logging.getLogger()
        for handler in list(root.handlers):
            if isinstance(handler, logging.handlers.RotatingFileHandler):
                root.removeHandler(handler)
                handler.close()
        logging_setup._configured = False


if __name__ == "__main__":
    unittest.main()
