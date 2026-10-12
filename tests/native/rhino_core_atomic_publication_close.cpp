// SPDX-License-Identifier: LGPL-2.1-or-later
// Native witnesses: FreeCAD owns journals and application-level close signals.
#include <gtest/gtest.h>
#include <string>
#include <stdexcept>
#include <vector>

#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/Transactions.h>
#include <Base/Exception.h>
#include <src/App/InitApplication.h>

class AtomicPublicationCloseTest : public ::testing::Test
{
protected:
    static void SetUpTestSuite() { tests::initApplication(); }
    void TearDown() override
    {
        for (const auto& name : names) {
            App::GetApplication().closeDocument(name.c_str());
        }
    }
    App::Document* document(const char* prefix)
    {
        auto& app = App::GetApplication();
        names.push_back(app.getUniqueDocumentName(prefix));
        return app.newDocument(names.back().c_str());
    }
    App::DocumentObject* source(App::Document& doc)
    {
        auto* object = doc.addObject("App::FeaturePython", "Source");
        object->Label.setValue("Initial");
        doc.clearUndos();
        return object;
    }
    void redoWitness(App::Document& doc, App::DocumentObject& object)
    {
        doc.openTransaction("Prior action");
        object.Label.setValue("Prior value");
        doc.commitTransaction();
        ASSERT_TRUE(doc.undo());
        ASSERT_EQ(doc.getAvailableRedos(), 1);
    }
    void lateParticipantWitness(bool abort)
    {
        auto& app = App::GetApplication();
        auto* first = document("ClosingMember");
        auto* unrelated = document("LateMember");
        auto* a = source(*first);
        auto* b = source(*unrelated);
        redoWitness(*unrelated, *b);
        const int priorRedo = unrelated->getTransactionID(false);
        const int id = first->openTransactionPreservingRedo("Close captured members");
        ASSERT_NE(id, 0);
        a->Label.setValue("Published");
        int joined = -1;
        auto before = app.signalBeforeCloseTransaction.connect([&](bool) {
            joined = unrelated->openTransaction("Late participant", id);
            if (joined != 0) {
                b->Label.setValue("Orphaned change");
            }
        });
        EXPECT_TRUE(abort ? app.abortTransaction(id) : app.commitTransaction(id));
        before.disconnect();
        EXPECT_EQ(joined, 0);
        EXPECT_EQ(unrelated->getBookedTransactionID(), 0);
        EXPECT_FALSE(unrelated->hasPendingTransaction());
        EXPECT_STREQ(b->Label.getValue(), "Initial");
        EXPECT_EQ(unrelated->getTransactionID(false), priorRedo);
        EXPECT_EQ(unrelated->getAvailableRedos(), 1);
        ASSERT_TRUE(unrelated->redo());
        EXPECT_STREQ(b->Label.getValue(), "Prior value");
        EXPECT_STREQ(a->Label.getValue(), abort ? "Initial" : "Published");
    }
    void emptyCommitHistoryWitness(bool mixed)
    {
        auto& app = App::GetApplication();
        auto* empty = document("EmptyHistory");
        auto* object = source(*empty);
        App::Document* materialized = mixed ? document("MaterializedHistory") : nullptr;
        App::DocumentObject* other = mixed ? source(*materialized) : nullptr;
        empty->setMaxUndoStackSize(10);
        const int oldest = empty->openTransaction("First prior edit");
        object->Label.setValue("First prior value");
        empty->commitTransaction();
        const int newest = empty->openTransaction("Second prior edit");
        object->Label.setValue("Second prior value");
        empty->commitTransaction();
        ASSERT_EQ(empty->getAvailableUndos(), 2);
        empty->setMaxUndoStackSize(1);
        const int id = (mixed ? materialized : empty)->openTransactionPreservingRedo("Empty publication");
        ASSERT_NE(id, 0);
        if (mixed) {
            ASSERT_EQ(empty->openTransaction("Empty participant", id), id);
            other->Label.setValue("Published");
        }
        ASSERT_FALSE(empty->hasPendingTransaction());
        EXPECT_TRUE(app.commitTransaction(id));
        EXPECT_EQ(empty->getBookedTransactionID(), 0);
        EXPECT_FALSE(empty->hasPendingTransaction());
        EXPECT_EQ(empty->getAvailableUndos(), 2);
        EXPECT_EQ(empty->getTransactionID(true, 0), newest);
        EXPECT_EQ(empty->getTransactionID(true, 1), oldest);
        ASSERT_TRUE(empty->undo());
        EXPECT_STREQ(object->Label.getValue(), "First prior value");
        ASSERT_TRUE(empty->undo());
        EXPECT_STREQ(object->Label.getValue(), "Initial");
        ASSERT_TRUE(empty->redo());
        EXPECT_STREQ(object->Label.getValue(), "First prior value");
        ASSERT_TRUE(empty->redo());
        EXPECT_STREQ(object->Label.getValue(), "Second prior value");
        if (mixed) {
            EXPECT_EQ(materialized->getAvailableUndos(), 1);
            ASSERT_TRUE(materialized->undo());
            EXPECT_STREQ(other->Label.getValue(), "Initial");
        }
    }
    void deletionObserverWitness(bool transactionSignal)
    {
        auto& app = App::GetApplication();
        auto* doc = document("DeleteObserver");
        auto* untouched = source(*doc);
        redoWitness(*doc, *untouched);
        const int oldRedo = doc->getTransactionID(false);
        const int id = doc->openTransactionPreservingRedo("Cancel created object");
        ASSERT_NE(id, 0);
        auto* temporary = doc->addObject("App::FeaturePython", "Temporary");
        bool observed = false;
        bool rejected = false;
        auto mutate = [&](const App::DocumentObject& object) {
            if (&object != temporary) {
                return;
            }
            observed = true;
            try { untouched->Label.setValue("Delete observer overwrite"); }
            catch (const Base::Exception&) { rejected = true; }
        };
        decltype(doc->signalDeletedObject.connect(mutate)) observer;
        if (transactionSignal) {
            observer = doc->signalTransactionRemove.connect(
                [&](const App::DocumentObject& object, App::Transaction*) { mutate(object); });
        }
        else {
            observer = doc->signalDeletedObject.connect(mutate);
        }
        EXPECT_TRUE(app.abortTransaction(id));
        observer.disconnect();
        EXPECT_TRUE(observed);
        EXPECT_TRUE(rejected);
        EXPECT_EQ(doc->getObject("Temporary"), nullptr);
        EXPECT_EQ(doc->getObject("Source"), untouched);
        EXPECT_STREQ(untouched->Label.getValue(), "Initial");
        EXPECT_FALSE(doc->hasPendingTransaction());
        EXPECT_EQ(doc->getAvailableUndos(), 0);
        EXPECT_EQ(doc->getTransactionID(false), oldRedo);
        ASSERT_EQ(doc->getAvailableRedos(), 1);
        ASSERT_TRUE(doc->redo());
        EXPECT_STREQ(untouched->Label.getValue(), "Prior value");
    }
    void nestedGlobalMoveWitness(bool sourceFirst)
    {
        auto& app = App::GetApplication();
        auto* outer = document("GlobalMoveOuter");
        auto* first = document("GlobalMoveA");
        auto* second = document("zzGlobalMoveB");
        auto* outerSource = source(*outer);
        auto* a = source(*first);
        auto* b = source(*second);
        auto* original = sourceFirst ? a : b;
        auto* receiver = sourceFirst ? b : a;
        auto* property = dynamic_cast<App::PropertyString*>(
            original->addDynamicProperty("App::PropertyString", "Moved"));
        ASSERT_NE(property, nullptr);
        property->setValue("Preserved value");
        first->clearUndos();
        second->clearUndos();
        redoWitness(*first, *a);
        redoWitness(*second, *b);
        const int firstRedo = first->getTransactionID(false);
        const int secondRedo = second->getTransactionID(false);
        const int innerID = first->openTransactionPreservingRedo("Nested property move");
        ASSERT_NE(innerID, 0);
        ASSERT_EQ(second->openTransaction("Move participant", innerID), innerID);
        ASSERT_NE(original->moveDynamicProperty(property, receiver), nullptr);
        const int globalID = app.openGlobalTransaction({.name = "Outer global edit", .temporary = false});
        ASSERT_NE(globalID, 0);
        ASSERT_EQ(outer->openTransaction("Outer participant", globalID), globalID);
        outerSource->Label.setValue("Published outer");
        bool entered = false;
        bool nestedResult = false;
        auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
            if (!abort && app.getGlobalTransaction() == globalID) {
                entered = true;
                nestedResult = app.abortTransaction(innerID);
            }
        });
        const int replacement = outer->openTransactionPreservingRedo("Next outer operation");
        before.disconnect();
        EXPECT_TRUE(entered);
        EXPECT_TRUE(nestedResult);
        ASSERT_NE(replacement, 0);
        EXPECT_TRUE(app.abortTransaction(replacement));
        auto* restored = dynamic_cast<App::PropertyString*>(original->getDynamicPropertyByName("Moved"));
        ASSERT_NE(restored, nullptr);
        EXPECT_EQ(restored->getContainer(), original);
        EXPECT_STREQ(restored->getValue(), "Preserved value");
        EXPECT_EQ(receiver->getDynamicPropertyByName("Moved"), nullptr);
        EXPECT_FALSE(first->hasPendingTransaction());
        EXPECT_FALSE(second->hasPendingTransaction());
        EXPECT_EQ(first->getBookedTransactionID(), 0);
        EXPECT_EQ(second->getBookedTransactionID(), 0);
        EXPECT_EQ(first->getTransactionID(false), firstRedo);
        EXPECT_EQ(second->getTransactionID(false), secondRedo);
        ASSERT_EQ(first->getAvailableRedos(), 1);
        ASSERT_EQ(second->getAvailableRedos(), 1);
        ASSERT_TRUE(first->redo());
        ASSERT_TRUE(second->redo());
        EXPECT_STREQ(a->Label.getValue(), "Prior value");
        EXPECT_STREQ(b->Label.getValue(), "Prior value");
        EXPECT_STREQ(outerSource->Label.getValue(), "Published outer");
    }
    std::vector<std::string> names;
};

TEST_F(AtomicPublicationCloseTest, BeforeCommitRejectsLateParticipant)
{
    lateParticipantWitness(false);
}

TEST_F(AtomicPublicationCloseTest, BeforeAbortRejectsLateParticipant)
{
    lateParticipantWitness(true);
}

TEST_F(AtomicPublicationCloseTest, EmptyCommitRetainsExecutablePriorUndo)
{
    emptyCommitHistoryWitness(false);
}

TEST_F(AtomicPublicationCloseTest, MixedCommitRetainsEmptyParticipantsPriorUndo)
{
    emptyCommitHistoryWitness(true);
}

TEST_F(AtomicPublicationCloseTest, DefaultAbortKeepsGlobalBeforeClosePair)
{
    auto& app = App::GetApplication();
    auto* doc = document("DefaultClose");
    auto* object = source(*doc);
    const int id = doc->openTransaction("Default edit");
    ASSERT_NE(id, 0);
    object->Label.setValue("Temporary");
    std::vector<std::string> events;
    bool abortArguments = true;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
        abortArguments = abortArguments && abort;
        events.push_back("before");
    });
    auto closed = app.signalCloseTransaction.connect([&](bool abort) {
        abortArguments = abortArguments && abort;
        events.push_back("close");
    });
    EXPECT_TRUE(app.abortTransaction(id));
    before.disconnect();
    closed.disconnect();
    EXPECT_TRUE(abortArguments);
    EXPECT_EQ(events, (std::vector<std::string>{"before", "close"}));
    EXPECT_STREQ(object->Label.getValue(), "Initial");
    EXPECT_FALSE(doc->hasPendingTransaction());
}

TEST_F(AtomicPublicationCloseTest, AbortSignalsOnceAfterAllParticipants)
{
    auto& app = App::GetApplication();
    auto* first = document("AtomicCloseA");
    auto* second = document("AtomicCloseB");
    auto* a = source(*first);
    auto* b = source(*second);
    redoWitness(*first, *a);
    redoWitness(*second, *b);
    const int redoA = first->getTransactionID(false);
    const int redoB = second->getTransactionID(false);
    const int id = first->openTransactionPreservingRedo("Atomic group edit");
    ASSERT_NE(id, 0);
    ASSERT_EQ(second->openTransaction("Participant", id), id);
    a->Label.setValue("Temporary A");
    b->Label.setValue("Temporary B");
    std::vector<std::string> events;
    bool abortArguments = true;
    bool beforePending = false;
    bool closeAfterRollback = false;
    bool closeBeforeRedoRestore = false;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
        abortArguments = abortArguments && abort;
        beforePending = first->hasPendingTransaction() && second->hasPendingTransaction()
            && first->getBookedTransactionID() == id && second->getBookedTransactionID() == id;
        events.push_back("before");
    });
    auto abortA = first->signalAbortTransaction.connect([&](const App::Document&) { events.push_back("abortA"); });
    auto stableA = first->signalBecameStable.connect([&](const App::Document&) { events.push_back("stableA"); });
    auto abortB = second->signalAbortTransaction.connect([&](const App::Document&) { events.push_back("abortB"); });
    auto stableB = second->signalBecameStable.connect([&](const App::Document&) { events.push_back("stableB"); });
    auto closed = app.signalCloseTransaction.connect([&](bool abort) {
        abortArguments = abortArguments && abort;
        closeAfterRollback = !first->hasPendingTransaction() && !second->hasPendingTransaction()
            && std::string(a->Label.getValue()) == "Initial" && std::string(b->Label.getValue()) == "Initial";
        closeBeforeRedoRestore = first->getAvailableRedos() == 0 && second->getAvailableRedos() == 0;
        events.push_back("close");
    });
    EXPECT_TRUE(app.abortTransaction(id));
    before.disconnect();
    abortA.disconnect();
    stableA.disconnect();
    abortB.disconnect();
    stableB.disconnect();
    closed.disconnect();
    EXPECT_TRUE(abortArguments);
    EXPECT_TRUE(beforePending);
    EXPECT_TRUE(closeAfterRollback);
    EXPECT_TRUE(closeBeforeRedoRestore);
    EXPECT_EQ(events, (std::vector<std::string>{"before", "abortA", "stableA", "abortB", "stableB", "close"}));
    EXPECT_EQ(first->getTransactionID(false), redoA);
    EXPECT_EQ(second->getTransactionID(false), redoB);
    EXPECT_EQ(first->getObject("Source"), a);
    EXPECT_EQ(second->getObject("Source"), b);
    ASSERT_TRUE(first->redo());
    EXPECT_EQ(first->getObject("Source"), a);
    EXPECT_STREQ(a->Label.getValue(), "Prior value");
    ASSERT_TRUE(second->redo());
    EXPECT_EQ(second->getObject("Source"), b);
    EXPECT_STREQ(b->Label.getValue(), "Prior value");
}

TEST_F(AtomicPublicationCloseTest, BeforeCommitCannotOverwriteValidatedSource)
{
    auto& app = App::GetApplication();
    auto* doc = document("AtomicCommit");
    auto* object = source(*doc);
    const int id = doc->openTransactionPreservingRedo("Publish validated edit");
    ASSERT_NE(id, 0);
    object->Label.setValue("Validated value");
    bool observed = false;
    bool rejected = false;
    bool beforeBooked = false;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
        if (abort) {
            return;
        }
        observed = true;
        beforeBooked = doc->getBookedTransactionID() == id && app.transactionIsActive(id);
        try {
            object->Label.setValue("Late observer value");
        }
        catch (const Base::Exception&) {
            rejected = true;
        }
    });
    EXPECT_TRUE(app.commitTransaction(id));
    before.disconnect();
    EXPECT_TRUE(observed);
    EXPECT_TRUE(rejected);
    EXPECT_TRUE(beforeBooked);
    EXPECT_STREQ(object->Label.getValue(), "Validated value");
    EXPECT_FALSE(doc->hasPendingTransaction());
    ASSERT_TRUE(doc->undo());
    EXPECT_STREQ(object->Label.getValue(), "Initial");
    ASSERT_TRUE(doc->redo());
    EXPECT_STREQ(object->Label.getValue(), "Validated value");
}

TEST_F(AtomicPublicationCloseTest, CommitCloseSeesCommittedGroupBeforeHistoryRetirement)
{
    auto& app = App::GetApplication();
    auto* doc = document("CommittedClose");
    auto* object = source(*doc);
    redoWitness(*doc, *object);
    const int id = doc->openTransactionPreservingRedo("Publish edit");
    object->Label.setValue("Published");
    bool observed = false;
    bool committed = false;
    bool rejected = false;
    auto closed = app.signalCloseTransaction.connect([&](bool abort) {
        observed = !abort;
        committed = !doc->hasPendingTransaction() && doc->getTransactionID(true) == id
            && doc->getBookedTransactionID() == 0 && !app.transactionIsActive(id);
        try { object->Label.setValue("Late close value"); }
        catch (const Base::Exception&) { rejected = true; }
    });
    EXPECT_TRUE(app.commitTransaction(id));
    closed.disconnect();
    EXPECT_TRUE(observed);
    EXPECT_TRUE(committed);
    EXPECT_TRUE(rejected);
    EXPECT_EQ(doc->getAvailableRedos(), 0);
    EXPECT_STREQ(object->Label.getValue(), "Published");
}

TEST_F(AtomicPublicationCloseTest, BeforeCommitExceptionRollsBackEveryParticipant)
{
    auto& app = App::GetApplication();
    auto* first = document("BeforeFailureA");
    auto* second = document("BeforeFailureB");
    auto* a = source(*first);
    auto* b = source(*second);
    redoWitness(*first, *a);
    redoWitness(*second, *b);
    const int id = first->openTransactionPreservingRedo("Fail before commit");
    ASSERT_EQ(second->openTransaction("Participant", id), id);
    a->Label.setValue("Temporary A");
    b->Label.setValue("Temporary B");
    auto before = app.signalBeforeCloseTransaction.connect([](bool) { throw std::runtime_error("Before observer failure"); });
    bool returned = false;
    bool threw = false;
    try { returned = app.commitTransaction(id); }
    catch (...) { threw = true; }
    before.disconnect();
    EXPECT_FALSE(threw);
    EXPECT_FALSE(returned);
    EXPECT_FALSE(first->hasPendingTransaction());
    EXPECT_FALSE(second->hasPendingTransaction());
    EXPECT_EQ(first->getBookedTransactionID(), 0);
    EXPECT_EQ(second->getBookedTransactionID(), 0);
    EXPECT_STREQ(a->Label.getValue(), "Initial");
    EXPECT_STREQ(b->Label.getValue(), "Initial");
    EXPECT_EQ(first->getAvailableUndos(), 0);
    EXPECT_EQ(second->getAvailableUndos(), 0);
    EXPECT_EQ(first->getAvailableRedos(), 0);
    EXPECT_EQ(second->getAvailableRedos(), 0);
    // A throwing before notification must not poison later default pairs.
    std::vector<std::string> events;
    auto recoveryBefore = app.signalBeforeCloseTransaction.connect([&](bool) { events.push_back("before"); });
    auto recoveryClose = app.signalCloseTransaction.connect([&](bool) { events.push_back("close"); });
    const int recovered = first->openTransaction("Recovery default edit");
    a->Label.setValue("Recovered");
    EXPECT_TRUE(app.commitTransaction(recovered));
    recoveryBefore.disconnect();
    recoveryClose.disconnect();
    EXPECT_EQ(events, (std::vector<std::string>{"before", "close"}));
    EXPECT_STREQ(a->Label.getValue(), "Recovered");
}

TEST_F(AtomicPublicationCloseTest, CloseCommitExceptionRollsBackStagedNativeJournals)
{
    auto& app = App::GetApplication();
    auto* first = document("CloseFailureA");
    auto* second = document("CloseFailureB");
    auto* a = source(*first);
    auto* b = source(*second);
    redoWitness(*first, *a);
    redoWitness(*second, *b);
    const int id = first->openTransactionPreservingRedo("Fail commit close");
    ASSERT_EQ(second->openTransaction("Participant", id), id);
    a->Label.setValue("Temporary A");
    b->Label.setValue("Temporary B");
    auto closed = app.signalCloseTransaction.connect([](bool) { throw std::runtime_error("Close observer failure"); });
    EXPECT_FALSE(app.commitTransaction(id));
    closed.disconnect();
    EXPECT_FALSE(first->hasPendingTransaction());
    EXPECT_FALSE(second->hasPendingTransaction());
    EXPECT_STREQ(a->Label.getValue(), "Initial");
    EXPECT_STREQ(b->Label.getValue(), "Initial");
    EXPECT_EQ(first->getAvailableUndos(), 0);
    EXPECT_EQ(second->getAvailableUndos(), 0);
    EXPECT_EQ(first->getAvailableRedos(), 0);
    EXPECT_EQ(second->getAvailableRedos(), 0);
}

TEST_F(AtomicPublicationCloseTest, BeforeAbortExceptionStillRollsBackEveryParticipant)
{
    auto& app = App::GetApplication();
    auto* doc = document("BeforeAbortFailure");
    auto* object = source(*doc);
    redoWitness(*doc, *object);
    const int id = doc->openTransactionPreservingRedo("Abort despite observer");
    object->Label.setValue("Temporary");
    auto before = app.signalBeforeCloseTransaction.connect([](bool) { throw std::runtime_error("Before abort observer failure"); });
    EXPECT_FALSE(app.abortTransaction(id));
    before.disconnect();
    EXPECT_STREQ(object->Label.getValue(), "Initial");
    EXPECT_FALSE(doc->hasPendingTransaction());
    EXPECT_EQ(doc->getAvailableRedos(), 0);
}

TEST_F(AtomicPublicationCloseTest, CloseAbortExceptionDisposesHeldRedoBeforeGroupRelease)
{
    auto& app = App::GetApplication();
    auto* doc = document("CloseAbortFailure");
    auto* object = source(*doc);
    redoWitness(*doc, *object);
    const int id = doc->openTransactionPreservingRedo("Abort with close failure");
    object->Label.setValue("Temporary");
    auto closed = app.signalCloseTransaction.connect([](bool) { throw std::runtime_error("Close abort observer failure"); });
    EXPECT_FALSE(app.abortTransaction(id));
    closed.disconnect();
    EXPECT_STREQ(object->Label.getValue(), "Initial");
    EXPECT_FALSE(doc->hasPendingTransaction());
    EXPECT_EQ(doc->getAvailableRedos(), 0);
}

TEST_F(AtomicPublicationCloseTest, NestedAbortHasIndependentGlobalNotificationPair)
{
    auto& app = App::GetApplication();
    auto* outer = document("NestedOuter");
    auto* inner = document("NestedInner");
    auto* a = source(*outer);
    auto* b = source(*inner);
    redoWitness(*inner, *b);
    const int oldRedo = inner->getTransactionID(false);
    const int outerID = outer->openTransactionPreservingRedo("Outer publication");
    const int innerID = inner->openTransactionPreservingRedo("Inner cancellation");
    a->Label.setValue("Published");
    b->Label.setValue("Temporary");
    bool nestedResult = false;
    std::vector<std::string> events;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
        events.push_back(abort ? "before-inner" : "before-outer");
        if (!abort) {
            nestedResult = app.abortTransaction(innerID);
        }
    });
    auto closed = app.signalCloseTransaction.connect([&](bool abort) {
        events.push_back(abort ? "close-inner" : "close-outer");
    });
    EXPECT_TRUE(app.commitTransaction(outerID));
    before.disconnect();
    closed.disconnect();
    EXPECT_TRUE(nestedResult);
    EXPECT_EQ(events, (std::vector<std::string>{"before-outer", "before-inner", "close-inner", "close-outer"}));
    EXPECT_STREQ(a->Label.getValue(), "Published");
    EXPECT_STREQ(b->Label.getValue(), "Initial");
    EXPECT_EQ(inner->getTransactionID(false), oldRedo);
    ASSERT_EQ(inner->getAvailableRedos(), 1);
    ASSERT_TRUE(inner->redo());
    EXPECT_STREQ(b->Label.getValue(), "Prior value");
}

TEST_F(AtomicPublicationCloseTest, NestedBeforeExceptionHasIndependentFailureVerdict)
{
    auto& app = App::GetApplication();
    auto* outer = document("NestedFailureOuter");
    auto* inner = document("NestedFailureInner");
    auto* a = source(*outer);
    auto* b = source(*inner);
    redoWitness(*inner, *b);
    const int outerID = outer->openTransactionPreservingRedo("Outer publication");
    const int innerID = inner->openTransactionPreservingRedo("Inner cancellation");
    a->Label.setValue("Published");
    b->Label.setValue("Temporary");
    bool nestedResult = true;
    int innerBeforeCount = 0;
    int innerCloseCount = 0;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
        if (abort) {
            ++innerBeforeCount;
            throw std::runtime_error("Nested before failure");
        }
        nestedResult = app.abortTransaction(innerID);
    });
    auto closed = app.signalCloseTransaction.connect([&](bool abort) {
        if (abort) {
            ++innerCloseCount;
        }
    });
    EXPECT_TRUE(app.commitTransaction(outerID));
    before.disconnect();
    closed.disconnect();
    EXPECT_FALSE(nestedResult);
    EXPECT_EQ(innerBeforeCount, 1);
    EXPECT_EQ(innerCloseCount, 1);
    EXPECT_FALSE(inner->hasPendingTransaction());
    EXPECT_EQ(inner->getBookedTransactionID(), 0);
    EXPECT_STREQ(b->Label.getValue(), "Initial");
    EXPECT_EQ(inner->getAvailableUndos(), 0);
    EXPECT_EQ(inner->getAvailableRedos(), 0);
    EXPECT_STREQ(a->Label.getValue(), "Published");
    ASSERT_TRUE(outer->undo());
    EXPECT_STREQ(a->Label.getValue(), "Initial");
}

TEST_F(AtomicPublicationCloseTest, ProtectedGlobalCloseRejectsAutomaticLateAdmission)
{
    auto& app = App::GetApplication();
    auto* first = document("GlobalClosingMember");
    auto* unrelated = document("GlobalLateMember");
    auto* a = source(*first);
    auto* b = source(*unrelated);
    redoWitness(*unrelated, *b);
    const int oldRedo = unrelated->getTransactionID(false);
    const int globalID = app.openGlobalTransaction({.name = "Prior global edit", .temporary = false});
    ASSERT_NE(globalID, 0);
    ASSERT_EQ(first->openTransaction("Prior participant", globalID), globalID);
    a->Label.setValue("Prior published");
    bool beforeSawGlobal = false;
    bool rejected = false;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool) {
        beforeSawGlobal = app.getGlobalTransaction() == globalID;
        try { b->Label.setValue("Automatic orphan"); }
        catch (const Base::Exception&) { rejected = true; }
    });
    const int replacement = first->openTransactionPreservingRedo("Next publication");
    before.disconnect();
    EXPECT_TRUE(beforeSawGlobal);
    EXPECT_TRUE(rejected);
    EXPECT_NE(replacement, 0);
    EXPECT_NE(replacement, globalID);
    EXPECT_EQ(unrelated->getBookedTransactionID(), 0);
    EXPECT_FALSE(unrelated->hasPendingTransaction());
    EXPECT_STREQ(b->Label.getValue(), "Initial");
    EXPECT_EQ(unrelated->getTransactionID(false), oldRedo);
    ASSERT_EQ(unrelated->getAvailableRedos(), 1);
    ASSERT_TRUE(unrelated->redo());
    EXPECT_STREQ(b->Label.getValue(), "Prior value");
    EXPECT_STREQ(a->Label.getValue(), "Prior published");
    EXPECT_TRUE(app.abortTransaction(replacement));
}

TEST_F(AtomicPublicationCloseTest, InnerCommitObserverCannotJoinClosingOuterGroup)
{
    auto& app = App::GetApplication();
    auto* outer = document("AdmissionOuter");
    auto* inner = document("AdmissionInner");
    auto* late = document("AdmissionLate");
    auto* a = source(*outer);
    auto* b = source(*inner);
    auto* c = source(*late);
    redoWitness(*late, *c);
    const int oldRedo = late->getTransactionID(false);
    const int outerID = outer->openTransactionPreservingRedo("Outer publication");
    const int innerID = inner->openTransaction("Separate inner publication");
    a->Label.setValue("Outer published");
    b->Label.setValue("Inner published");
    bool entered = false;
    bool innerResult = false;
    int joined = -1;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
        if (!abort && !entered) {
            entered = true;
            innerResult = app.commitTransaction(innerID);
        }
    });
    auto innerCommit = inner->signalCommitTransaction.connect([&](const App::Document&) {
        joined = late->openTransaction("Late outer participant", outerID);
        if (joined != 0) {
            c->Label.setValue("Nested orphan");
        }
    });
    EXPECT_TRUE(app.commitTransaction(outerID));
    before.disconnect();
    innerCommit.disconnect();
    EXPECT_TRUE(entered);
    EXPECT_TRUE(innerResult);
    EXPECT_EQ(joined, 0);
    EXPECT_EQ(late->getBookedTransactionID(), 0);
    EXPECT_FALSE(late->hasPendingTransaction());
    EXPECT_STREQ(c->Label.getValue(), "Initial");
    EXPECT_EQ(late->getTransactionID(false), oldRedo);
    ASSERT_EQ(late->getAvailableRedos(), 1);
    ASSERT_TRUE(late->redo());
    EXPECT_STREQ(c->Label.getValue(), "Prior value");
    EXPECT_STREQ(a->Label.getValue(), "Outer published");
    EXPECT_STREQ(b->Label.getValue(), "Inner published");
}

TEST_F(AtomicPublicationCloseTest, RestoredObjectObserverCannotOverwriteUntouchedSource)
{
    auto& app = App::GetApplication();
    auto* doc = document("RestoreObserver");
    auto* untouched = source(*doc);
    auto* victim = doc->addObject("App::FeaturePython", "Victim");
    const auto victimID = victim->getID();
    doc->clearUndos();
    redoWitness(*doc, *untouched);
    const int oldRedo = doc->getTransactionID(false);
    const int id = doc->openTransactionPreservingRedo("Cancel object deletion");
    ASSERT_NE(id, 0);
    doc->removeObject("Victim");
    bool observed = false;
    bool rejected = false;
    auto observer = doc->signalNewObject.connect([&](const App::DocumentObject& object) {
        if (&object != victim) {
            return;
        }
        observed = true;
        try { untouched->Label.setValue("Restore observer overwrite"); }
        catch (const Base::Exception&) { rejected = true; }
    });
    EXPECT_TRUE(app.abortTransaction(id));
    observer.disconnect();
    EXPECT_TRUE(observed);
    EXPECT_TRUE(rejected);
    EXPECT_EQ(doc->getObject("Victim"), victim);
    EXPECT_EQ(doc->getObjectByID(victimID), victim);
    EXPECT_STREQ(untouched->Label.getValue(), "Initial");
    EXPECT_FALSE(doc->hasPendingTransaction());
    EXPECT_EQ(doc->getAvailableUndos(), 0);
    EXPECT_EQ(doc->getTransactionID(false), oldRedo);
    ASSERT_EQ(doc->getAvailableRedos(), 1);
    ASSERT_TRUE(doc->redo());
    EXPECT_STREQ(untouched->Label.getValue(), "Prior value");
    EXPECT_EQ(doc->getObjectByID(victimID), victim);
}

TEST_F(AtomicPublicationCloseTest, DeletedObjectObserverCannotOverwriteUntouchedSource)
{
    deletionObserverWitness(false);
}

TEST_F(AtomicPublicationCloseTest, ActivatedRestoredObjectObserverCannotOverwriteUntouchedSource)
{
    auto& app = App::GetApplication();
    auto* doc = document("ActivateRestoreObserver");
    auto* untouched = source(*doc);
    auto* victim = doc->addObject("App::FeaturePython", "Victim");
    const auto victimID = victim->getID();
    doc->clearUndos();
    redoWitness(*doc, *untouched);
    const int oldRedo = doc->getTransactionID(false);
    const int id = doc->openTransactionPreservingRedo("Cancel object deletion with activation");
    ASSERT_NE(id, 0);
    doc->removeObject("Victim");
    bool observed = false;
    bool rejected = false;
    auto observer = doc->signalActivatedObject.connect([&](const App::DocumentObject& object) {
        if (&object != victim) {
            return;
        }
        observed = true;
        try { untouched->Label.setValue("Activation observer overwrite"); }
        catch (const Base::Exception&) { rejected = true; }
    });
    EXPECT_TRUE(app.abortTransaction(id));
    observer.disconnect();
    EXPECT_TRUE(observed);
    EXPECT_TRUE(rejected);
    EXPECT_EQ(doc->getObject("Victim"), victim);
    EXPECT_EQ(doc->getObjectByID(victimID), victim);
    EXPECT_STREQ(untouched->Label.getValue(), "Initial");
    EXPECT_FALSE(doc->hasPendingTransaction());
    EXPECT_EQ(doc->getAvailableUndos(), 0);
    EXPECT_EQ(doc->getTransactionID(false), oldRedo);
    ASSERT_EQ(doc->getAvailableRedos(), 1);
    ASSERT_TRUE(doc->redo());
    EXPECT_STREQ(untouched->Label.getValue(), "Prior value");
    EXPECT_EQ(doc->getObjectByID(victimID), victim);
}

TEST_F(AtomicPublicationCloseTest, RemovalJournalObserverCannotOverwriteUntouchedSource)
{
    deletionObserverWitness(true);
}

TEST_F(AtomicPublicationCloseTest, NestedGlobalMoveRestoresSourceFirst)
{
    nestedGlobalMoveWitness(true);
}

TEST_F(AtomicPublicationCloseTest, NestedGlobalMoveRestoresReceiverFirst)
{
    nestedGlobalMoveWitness(false);
}

TEST_F(AtomicPublicationCloseTest, NestedRollbackFailureDoesNotPoisonIndependentOuterRollback)
{
    auto& app = App::GetApplication();
    auto* outer = document("RollbackVerdictOuter");
    auto* inner = document("RollbackVerdictInner");
    auto* a = source(*outer);
    auto* b = source(*inner);
    redoWitness(*outer, *a);
    redoWitness(*inner, *b);
    const int outerRedo = outer->getTransactionID(false);
    const int outerID = outer->openTransactionPreservingRedo("Outer cancellation");
    const int innerID = inner->openTransactionPreservingRedo("Separate inner cancellation");
    ASSERT_NE(outerID, innerID);
    a->Label.setValue("Temporary outer");
    b->Label.setValue("Temporary inner");
    bool entered = false;
    bool nestedResult = true;
    auto rejectInner = inner->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (&object == b && &property == &b->Label) {
                throw std::runtime_error("Independent inner rollback failed");
            }
        });
    auto nestDuringRollback = outer->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (!entered && &object == a && &property == &a->Label) {
                entered = true;
                nestedResult = app.abortTransaction(innerID);
            }
        });
    EXPECT_TRUE(app.abortTransaction(outerID));
    nestDuringRollback.disconnect();
    rejectInner.disconnect();
    EXPECT_TRUE(entered);
    EXPECT_FALSE(nestedResult);
    EXPECT_STREQ(a->Label.getValue(), "Initial");
    EXPECT_FALSE(outer->hasPendingTransaction());
    EXPECT_EQ(outer->getBookedTransactionID(), 0);
    EXPECT_EQ(outer->getTransactionID(false), outerRedo);
    ASSERT_EQ(outer->getAvailableRedos(), 1);
    ASSERT_TRUE(outer->redo());
    EXPECT_STREQ(a->Label.getValue(), "Prior value");
    EXPECT_STREQ(b->Label.getValue(), "Temporary inner");
    EXPECT_FALSE(inner->hasPendingTransaction());
    EXPECT_EQ(inner->getBookedTransactionID(), 0);
    EXPECT_EQ(inner->getAvailableRedos(), 0);
}

TEST_F(AtomicPublicationCloseTest, SameTransactionNestedRollbackFailureInvalidatesOuterVerdict)
{
    auto* outer = document("SameVerdictOuter");
    auto* inner = document("SameVerdictInner");
    auto* a = source(*outer);
    auto* b = source(*inner);
    const int sharedID = App::Transaction::getNewID();
    App::Transaction outerJournal(sharedID);
    App::Transaction innerJournal(sharedID);
    outerJournal.addObjectChange(a, &a->Label);
    innerJournal.addObjectChange(b, &b->Label);
    a->Label.setValue("Temporary outer");
    b->Label.setValue("Temporary inner");
    bool entered = false;
    bool nestedResult = true;
    auto rejectInner = inner->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (&object == b && &property == &b->Label) {
                throw std::runtime_error("Same transaction rollback failed");
            }
        });
    auto nestDuringRollback = outer->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (!entered && &object == a && &property == &a->Label) {
                entered = true;
                nestedResult = innerJournal.applyChecked(*inner, false);
            }
        });
    EXPECT_FALSE(outerJournal.applyChecked(*outer, false));
    nestDuringRollback.disconnect();
    rejectInner.disconnect();
    EXPECT_TRUE(entered);
    EXPECT_FALSE(nestedResult);
    EXPECT_STREQ(a->Label.getValue(), "Initial");
    EXPECT_STREQ(b->Label.getValue(), "Temporary inner");
}

TEST_F(AtomicPublicationCloseTest, OuterBeforeFailureKeepsCompletedIndependentInnerRedo)
{
    auto& app = App::GetApplication();
    auto* outer = document("OuterFailureCompletedInner");
    auto* inner = document("CompletedInner");
    auto* a = source(*outer);
    auto* b = source(*inner);
    redoWitness(*outer, *a);
    redoWitness(*inner, *b);
    const int innerRedo = inner->getTransactionID(false);
    const int outerID = outer->openTransactionPreservingRedo("Fail outer publication");
    const int innerID = inner->openTransactionPreservingRedo("Independent inner cancellation");
    a->Label.setValue("Temporary outer");
    b->Label.setValue("Temporary inner");
    bool entered = false;
    bool innerResult = false;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
        if (!abort && !entered) {
            entered = true;
            innerResult = app.abortTransaction(innerID);
            throw std::runtime_error("Outer before failed after inner completion");
        }
    });
    EXPECT_FALSE(app.commitTransaction(outerID));
    before.disconnect();
    EXPECT_TRUE(entered);
    EXPECT_TRUE(innerResult);
    EXPECT_STREQ(a->Label.getValue(), "Initial");
    EXPECT_EQ(outer->getAvailableRedos(), 0);
    EXPECT_FALSE(outer->hasPendingTransaction());
    EXPECT_STREQ(b->Label.getValue(), "Initial");
    EXPECT_FALSE(inner->hasPendingTransaction());
    EXPECT_EQ(inner->getTransactionID(false), innerRedo);
    ASSERT_EQ(inner->getAvailableRedos(), 1);
    ASSERT_TRUE(inner->redo());
    EXPECT_STREQ(b->Label.getValue(), "Prior value");
}

TEST_F(AtomicPublicationCloseTest, InnerCloseExceptionHasIndependentFailureVerdict)
{
    auto& app = App::GetApplication();
    auto* outer = document("OuterWithInnerCloseFailure");
    auto* inner = document("InnerCloseFailure");
    auto* a = source(*outer);
    auto* b = source(*inner);
    redoWitness(*inner, *b);
    const int outerID = outer->openTransactionPreservingRedo("Publish outer");
    const int innerID = inner->openTransactionPreservingRedo("Cancel inner");
    a->Label.setValue("Published outer");
    b->Label.setValue("Temporary inner");
    bool entered = false;
    bool innerResult = true;
    auto before = app.signalBeforeCloseTransaction.connect([&](bool abort) {
        if (!abort && !entered) {
            entered = true;
            innerResult = app.abortTransaction(innerID);
        }
    });
    auto closed = app.signalCloseTransaction.connect([](bool abort) {
        if (abort) {
            throw std::runtime_error("Independent inner close failed");
        }
    });
    EXPECT_TRUE(app.commitTransaction(outerID));
    before.disconnect();
    closed.disconnect();
    EXPECT_TRUE(entered);
    EXPECT_FALSE(innerResult);
    EXPECT_STREQ(b->Label.getValue(), "Initial");
    EXPECT_FALSE(inner->hasPendingTransaction());
    EXPECT_EQ(inner->getAvailableRedos(), 0);
    EXPECT_STREQ(a->Label.getValue(), "Published outer");
    ASSERT_EQ(outer->getAvailableUndos(), 1);
    ASSERT_TRUE(outer->undo());
    EXPECT_STREQ(a->Label.getValue(), "Initial");
}

TEST_F(AtomicPublicationCloseTest, SameTransactionFailureCrossesIndependentNestedVerdict)
{
    auto* outer = document("SharedVerdictOuter");
    auto* middle = document("IndependentVerdictMiddle");
    auto* inner = document("SharedVerdictInner");
    auto* a = source(*outer);
    auto* b = source(*middle);
    auto* c = source(*inner);
    const int sharedID = App::Transaction::getNewID();
    App::Transaction outerJournal(sharedID);
    App::Transaction middleJournal;
    App::Transaction innerJournal(sharedID);
    ASSERT_NE(middleJournal.getID(), sharedID);
    outerJournal.addObjectChange(a, &a->Label);
    middleJournal.addObjectChange(b, &b->Label);
    innerJournal.addObjectChange(c, &c->Label);
    a->Label.setValue("Temporary outer");
    b->Label.setValue("Temporary middle");
    c->Label.setValue("Temporary inner");
    bool enteredOuter = false;
    bool enteredMiddle = false;
    bool middleResult = false;
    bool innerResult = true;
    auto rejectInner = inner->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (&object == c && &property == &c->Label) {
                throw std::runtime_error("Shared transaction inner rollback failed");
            }
        });
    auto nestInMiddle = middle->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (!enteredMiddle && &object == b && &property == &b->Label) {
                enteredMiddle = true;
                innerResult = innerJournal.applyChecked(*inner, false);
            }
        });
    auto nestInOuter = outer->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (!enteredOuter && &object == a && &property == &a->Label) {
                enteredOuter = true;
                middleResult = middleJournal.applyChecked(*middle, false);
            }
        });
    EXPECT_FALSE(outerJournal.applyChecked(*outer, false));
    nestInOuter.disconnect();
    nestInMiddle.disconnect();
    rejectInner.disconnect();
    EXPECT_TRUE(enteredOuter);
    EXPECT_TRUE(enteredMiddle);
    EXPECT_TRUE(middleResult);
    EXPECT_FALSE(innerResult);
    EXPECT_STREQ(a->Label.getValue(), "Initial");
    EXPECT_STREQ(b->Label.getValue(), "Initial");
    EXPECT_STREQ(c->Label.getValue(), "Temporary inner");
}

TEST_F(AtomicPublicationCloseTest, DefaultNestedRollbackDoesNotBorrowOuterCheckedVerdict)
{
    auto& app = App::GetApplication();
    auto* outer = document("CheckedOuterDefaultInner");
    auto* inner = document("DefaultInnerRollback");
    auto* a = source(*outer);
    auto* b = source(*inner);
    redoWitness(*outer, *a);
    redoWitness(*inner, *b);
    const int outerRedo = outer->getTransactionID(false);
    const int outerID = outer->openTransactionPreservingRedo("Checked outer cancellation");
    const int innerID = inner->openTransaction("Default inner cancellation");
    ASSERT_NE(outerID, innerID);
    a->Label.setValue("Temporary outer");
    b->Label.setValue("Temporary inner");
    ASSERT_EQ(inner->getAvailableRedos(), 0);
    bool entered = false;
    bool innerResult = false;
    auto rejectInner = inner->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (&object == b && &property == &b->Label) {
                throw std::runtime_error("Default inner restoration failed");
            }
        });
    auto nestDuringRollback = outer->signalBeforeChangeObject.connect(
        [&](const App::DocumentObject& object, const App::Property& property) {
            if (!entered && &object == a && &property == &a->Label) {
                entered = true;
                innerResult = app.abortTransaction(innerID);
            }
        });
    EXPECT_TRUE(app.abortTransaction(outerID));
    nestDuringRollback.disconnect();
    rejectInner.disconnect();
    EXPECT_TRUE(entered);
    // The default API keeps its historical best-effort verdict. Its caught
    // failure must not inherit the independent outer checked transaction ID.
    EXPECT_TRUE(innerResult);
    EXPECT_STREQ(b->Label.getValue(), "Temporary inner");
    EXPECT_EQ(inner->getAvailableRedos(), 0);
    EXPECT_FALSE(inner->hasPendingTransaction());
    EXPECT_STREQ(a->Label.getValue(), "Initial");
    EXPECT_FALSE(outer->hasPendingTransaction());
    EXPECT_EQ(outer->getTransactionID(false), outerRedo);
    ASSERT_EQ(outer->getAvailableRedos(), 1);
    ASSERT_TRUE(outer->redo());
    EXPECT_STREQ(a->Label.getValue(), "Prior value");
}
